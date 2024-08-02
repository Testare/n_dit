use std::borrow::{Borrow, Cow};
use std::fs::File;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, Mutex};

use bevy::ecs::entity::EntityHashSet;
use bevy::ecs::reflect::AppTypeRegistry;
use bevy::ecs::schedule::ScheduleLabel;
use bevy::scene::{ron, SceneFilter};
use freeform::SerdeScheme;
use serde::Serialize;
use typed_key::Key;

use crate::op::{Op, OpError, OpErrorUtils, OpImplResult, OpPlugin};
use crate::prelude::*;

mod key {
    use typed_key::{typed_key, Key};

    pub const SCENE: Key<String> = typed_key!("scene");
}

#[derive(Debug)]
pub struct SavePlugin;

impl Plugin for SavePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CurrentSaveFile>()
            .init_resource::<SaveFilter>()
            .init_schedule(SaveSchedule)
            .init_schedule(LoadSchedule)
            .add_plugins(OpPlugin::<SaveOp>::default());
    }
}

#[derive(Clone, Debug, Resource)]
pub struct CurrentSaveFile(Cow<'static, Path>);

impl Default for CurrentSaveFile {
    fn default() -> Self {
        Self(Cow::Borrowed(Path::new("default.sav.json")))
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

#[derive(Debug, Resource)]
pub struct SaveData {
    send: Sender<SaveEvent>,
    recv: Mutex<Receiver<SaveEvent>>, // Mutex to be replaced with std::sync::Exclusive once that is no longer nightly exclusive
}

impl Default for SaveData {
    fn default() -> Self {
        let (send, recv) = std::sync::mpsc::channel();
        Self {
            send,
            recv: Mutex::new(recv),
        }
    }
}

impl SaveData {
    fn send(&self, event: SaveEvent) {
        self.send.send(event).expect("the receiver should never be disconnected as the SaveData API should not allow it except in consuming methods");
    }

    pub fn add_entities(&self, entities: &[Entity]) {
        self.send(SaveEvent::AddEntitiesToScene(entities.to_owned()))
    }
    /**
     * This function will pass the whole save data metadata to it, which can be potentially unsafe.
     *
     * Also, this does not guarantee that the save data is complete: Make sure that you order
     * systems in order to make sure the save is properly processed
     *
     */
    pub fn alter<F: FnOnce(&mut Metadata) + Send + 'static>(&self, func: F) {
        self.send(SaveEvent::AlterSave(Box::new(func)));
    }

    // Note: In the future we might want to actually save objects to the save metadata with
    // freeeform/Sord, and then we would have to take val not &val
    pub fn put<T: Serialize, B: Borrow<T>>(
        &self,
        key: Key<T>,
        val: B,
    ) -> Result<(), Arc<serde_json::Error>> {
        let val_s = freeform::Json::serialize(val.borrow())?;
        let n = SaveEvent::Put(key.name(), val_s);
        self.send(n);
        Ok(())
    }

    pub fn process(self, world: &mut World) -> Result<Metadata, ron::Error> {
        let SaveData { send: _, mut recv } = self;
        let mut metadata = Metadata::new();
        let mut entity_set = EntityHashSet::default();
        let recv = recv
            .get_mut()
            .expect("this should be the only place where the mutex is accessed");
        for event in recv.try_iter() {
            event.apply(&mut metadata, &mut entity_set)
        }
        let save_filter = world.resource::<SaveFilter>().deref().clone();
        let scene = bevy::scene::DynamicSceneBuilder::from_world(world)
            .with_filter(save_filter.clone())
            .with_resource_filter(save_filter)
            .extract_entities(entity_set.into_iter())
            .build();
        let registry = world.resource::<AppTypeRegistry>();
        let save_data = scene.serialize(&registry.read())?;
        metadata
            .put(key::SCENE, save_data)
            .expect("should be easy to serialize a string");
        // TODO serialize scene from entity_set
        Ok(metadata)
    }
}

enum SaveEvent {
    AddEntitiesToScene(Vec<Entity>),
    Put(&'static str, String),
    AlterSave(Box<dyn FnOnce(&mut Metadata) + Send>),
}

impl SaveEvent {
    fn apply(self, metadata: &mut Metadata, entity_hash_set: &mut EntityHashSet) {
        match self {
            Self::AlterSave(f) => {
                f(metadata);
            },
            Self::Put(key, value) => {
                unsafe {
                    // Event is only populated with valid JSON
                    metadata.put_field_directly(key, value);
                }
            },
            Self::AddEntitiesToScene(entities) => {
                entity_hash_set.extend(entities);
            },
        }
    }
}

impl std::fmt::Debug for SaveEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AddEntitiesToScene(entities) => {
                write!(f, "SaveEvent::AddEntitiesToScene({entities:?})")
            },
            Self::Put(key, value) => {
                write!(f, "SaveEvent::Put({key:?}, {value:?})")
            },
            Self::AlterSave(_) => {
                write!(f, "SaveEvent::AlterSave(?)")
            },
        }
    }
}

#[derive(Debug, Resource, Deref, DerefMut)]
pub struct SaveFilter(SceneFilter);

impl Default for SaveFilter {
    fn default() -> Self {
        Self(SceneFilter::deny_all())
    }
}

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
    if let Some(save_data) = world.remove_resource::<SaveData>() {
        // TODO return different metadata
        // TODO more importantly, this writing should be async instead of in-frame
        let save_metadata = save_data
            .process(world)
            .map_err(|e| format!("Problem serializing save file: {e:?}").critical())?;
        serde_json::to_writer(file, &save_metadata).critical()?;
        Ok(save_metadata)
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
