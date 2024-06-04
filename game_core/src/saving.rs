use std::borrow::Cow;
use std::fs::File;
use std::path::{Path, PathBuf};

use bevy::ecs::schedule::ScheduleLabel;

use crate::op::{Op, OpErrorUtils, OpImplResult, OpPlugin};
use crate::prelude::*;

pub mod key {
    use typed_key::{typed_key, Key};

    pub const LEMONS: Key<String> = typed_key!("lemons");
}

#[derive(Debug)]
pub struct SavePlugin;

impl Plugin for SavePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CurrentSaveFile>()
            .init_schedule(SaveSchedule)
            .add_systems(SaveSchedule, sys_save_test_flag)
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
                    return Err(std::io::Error::new(std::io::ErrorKind::Other, "No home directory"));
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
        if self.0.exists() {
            return File::create(&self.0);
        }
        let path = self.get_path()?;
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
}

#[derive(Component, Debug, Default, Resource, Deref, DerefMut)]
pub struct SaveMetadata(Metadata);

#[derive(Clone, Debug, Eq, Hash, PartialEq, ScheduleLabel)]
pub struct SaveSchedule;

#[derive(Debug, Clone, Reflect)]
pub struct SaveOp;

impl Op for SaveOp {
    fn register_systems(mut registrar: crate::op::OpRegistrar<Self>) {
        registrar.register_op_exclusive(opsys_save_op);
    }

    fn system_index(&self) -> usize {
        0
    }
}

pub fn opsys_save_op(
    In((_source, _op)): In<(Entity, SaveOp)>,
    world: &mut World
) -> OpImplResult {
    world.insert_resource(SaveMetadata::default());
    let current_save_file = world
        .get_resource::<CurrentSaveFile>()
        .cloned()
        .ok_or_else(|| "No save file configured".critical())?;
    let file = current_save_file.create().critical()?;
    world.run_schedule(SaveSchedule);
    if let Some(SaveMetadata(metadata)) = world.remove_resource::<SaveMetadata>() {
        // TODO save this metadata, don't just return it
        serde_json::to_writer(file, &metadata).critical()?;
        Ok(metadata)
    } else {
        Err("Something went wrong, unable to save".critical())
    }
}

pub fn sys_save_test_flag(
    mut res_save_data: ResMut<SaveMetadata>,
) {
    res_save_data.put(key::LEMONS, "Bag of bones".to_string());
}
