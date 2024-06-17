use std::borrow::Cow;
use std::fs::File;
use std::path::{Path, PathBuf};

use bevy::ecs::schedule::ScheduleLabel;

use crate::op::{Op, OpError, OpErrorUtils, OpImplResult, OpPlugin};
use crate::prelude::*;

#[derive(Debug)]
pub struct SavePlugin;

impl Plugin for SavePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CurrentSaveFile>()
            .init_schedule(SaveSchedule)
            .init_schedule(LoadSchedule)
            .add_plugins(OpPlugin::<SaveOp>::default());
    }
}

#[derive(Clone, Debug, Resource)]
pub struct CurrentSaveFile(Cow<'static, Path>);

impl Default for CurrentSaveFile {
    fn default() -> Self {
        Self(Cow::Borrowed(Path::new("default.cq.sav")))
    }
}

impl CurrentSaveFile {
    fn get_path(&self) -> std::io::Result<Cow<'static, Path>> {
        if self.0.parent() == Some(Path::new("")) {
            let mut path_buf = self.get_os_save_directory()?;
            path_buf.push(self.0.clone());
            Ok(Cow::Owned(path_buf))
        } else {
            Ok(self.0.clone())
        }
    }

    fn get_os_save_directory(&self) -> std::io::Result<PathBuf> {
        // TODO actually change based on compiled OS.
        let mut path = std::env::var("XDG_DATA_HOME")
            .map(PathBuf::from)
            .or_else(|_| {
                let mut pathbuf = PathBuf::new();
                let home = homedir::get_my_home()?;
                if home.is_none() {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        "No home directory",
                    ));
                }
                pathbuf.push(home.unwrap());
                pathbuf.push(".local");
                pathbuf.push("share");
                Ok(pathbuf)
            })?;
        path.push("nf"); // TODO make this configurable
        Ok(path)
    }

    fn create(&self) -> std::io::Result<File> {
        let path = self.get_path()?;
        if path.exists() {
            return File::create(path);
        }
        let parent = path
            .parent()
            .expect("There should always be a parent in the expanded path");
        if !parent.exists() {
            log::info!("Creating parent directory {parent:?}");
            std::fs::create_dir_all(parent)?;
        }
        log::info!("Creating save file {path:?}");
        File::create(path)
        // Err(std::io::Error::new(std::io::ErrorKind::Other, "This was just test"))
    }

    fn open(&self) -> std::io::Result<File> {
        File::open(self.get_path()?)
    }
}

/// Contains data from loaded save file, to provide as a resource
/// to the bevy LoadSchedule.
///
/// Essentially identical to [SaveData], but to prevent load/save systems
/// from running in the wrong schedules this is kept separate and you
/// can't deref it as mut.
#[derive(Debug, Default, Resource, Deref)]
pub struct LoadData(Metadata);

#[derive(Clone, Debug, Eq, Hash, PartialEq, ScheduleLabel)]
pub struct LoadSchedule;

#[derive(Debug, Default, Resource, Deref, DerefMut)]
pub struct SaveData(Metadata);

#[derive(Clone, Debug, Eq, Hash, PartialEq, ScheduleLabel)]
pub struct SaveSchedule;

// TODO when saving becomes more involved, break Load op into two ops: InitiateLoad (load to
// memory) and PerformLoad
#[derive(Debug, Clone, Reflect)]
pub enum SaveOp {
    Save,
    Load,
}

impl Op for SaveOp {
    fn register_systems(mut registrar: crate::op::OpRegistrar<Self>) {
        registrar
            .register_op_exclusive(opsys_save_op)
            .register_op_exclusive(opsys_load_op);
    }

    fn system_index(&self) -> usize {
        match self {
            Self::Save => 0,
            Self::Load => 1,
        }
    }
}

pub fn opsys_save_op(In((_source, op)): In<(Entity, SaveOp)>, world: &mut World) -> OpImplResult {
    if !matches!(op, SaveOp::Save) {
        return Err(OpError::MismatchedOpSystem);
    }
    world.insert_resource(SaveData::default());
    let current_save_file = world
        .get_resource::<CurrentSaveFile>()
        .cloned()
        .ok_or_else(|| "No save file configured".critical())?;
    let file = current_save_file.create().critical()?;
    world.run_schedule(SaveSchedule);
    if let Some(SaveData(metadata)) = world.remove_resource::<SaveData>() {
        // TODO return different metadata
        // TODO more importantly, this writing should be async instead of in-frame
        serde_json::to_writer(file, &metadata).critical()?;
        Ok(metadata)
    } else {
        Err("Something went wrong, unable to save".critical())
    }
}

pub fn opsys_load_op(In((_source, op)): In<(Entity, SaveOp)>, world: &mut World) -> OpImplResult {
    if !matches!(op, SaveOp::Load) {
        return Err(OpError::MismatchedOpSystem);
    }
    let current_save_file = world
        .get_resource::<CurrentSaveFile>()
        .cloned()
        .ok_or_else(|| "No save file configured".critical())?;
    let file = current_save_file.open().critical()?;
    let data: Metadata = serde_json::from_reader(file).critical()?;
    world.insert_resource(LoadData(data));
    world.run_schedule(LoadSchedule);
    world.remove_resource::<LoadData>();
    Ok(default())
}
