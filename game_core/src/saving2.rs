use std::sync::{mpsc::{Receiver, Sender}, Arc, Mutex};
use std::fmt::Debug;
use bevy::{ecs::{entity::EntityHashSet, reflect::AppTypeRegistry}, scene::{ron, DynamicSceneBuilder, SceneFilter}};
use freeform::SerdeScheme;
use serde::Serialize;
use typed_key::Key;

use crate::prelude::*;

mod key {
    use typed_key::{Key, typed_key};

    pub const SCENE: Key<String> = typed_key!("scene");
}

#[derive(Debug, Resource)]
pub struct SaveData {
    send: Sender<SaveEvent>, 
    recv: Mutex<Receiver<SaveEvent>>,  // Mutex to be replaced with std::sync::Exclusive once that is no longer nightly exclusive
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

    pub fn add_entry<T: Serialize>(&self, key: Key<T>, val: T) -> Result<(), Arc<serde_json::Error>> {
        let val_s = freeform::Json::serialize(&val)?;
        let n = SaveEvent::AddMetadata(key.name(), val_s);
        self.send(n);
        Ok(())
    }

    pub fn process(self, world: &mut World) -> Result<Metadata, ron::Error> {
        let SaveData { send: _, mut recv } = self;
        let mut metadata = Metadata::new();
        let mut entity_set = EntityHashSet::default();
        let recv = recv.get_mut().expect("this should be the only place where the mutex is accessed");
        for event in recv.try_iter()  {
            event.apply(&mut metadata, &mut entity_set)
        }
        let save_filter = world.resource::<SaveFilter>().deref().clone();
        let scene = DynamicSceneBuilder::from_world(world)
            .with_filter(save_filter.clone())
            .with_resource_filter(save_filter)
            .extract_entities(entity_set.into_iter())
            .build();
        let registry = world.resource::<AppTypeRegistry>();
        let save_data = scene.serialize(&registry.read())?;
        metadata.put(key::SCENE, save_data).expect("should be easy to serialize a string");
        // TODO serialize scene from entity_set
        Ok(metadata)
    }

}

enum SaveEvent {
    AddEntitiesToScene(Vec<Entity>), // TODO NOCOMMIT should this be EntityHashSet instead?
    AddMetadata(&'static str, String),
    AlterSave(Box<dyn FnOnce(&mut Metadata) + Send>)
}

impl SaveEvent {
    fn apply(self, metadata: &mut Metadata, entity_hash_set: &mut EntityHashSet) {
        match self {
            Self::AlterSave(f) => {
                f(metadata);
            },
            Self::AddMetadata(key, value) => {
                unsafe {
                    // Event is only populated with valid JSON
                    metadata.put_field_directly(key, value);
                }
            },
            Self::AddEntitiesToScene(entities) => {
                entity_hash_set.extend(entities);
            }
        }
    }
}

impl Debug for SaveEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AddEntitiesToScene(entities) => {
                write!(f, "SaveEvent::AddEntitiesToScene({entities:?})")
            },
            Self::AddMetadata(key, value) => {
                write!(f, "SaveEvent::AddMetadada({key:?}, {value:?})")
            },
            Self::AlterSave(_) => {
                write!(f, "SaveEvent::AlterSave(?)")
            },
        }
    }
}

#[derive(Debug, Resource, Deref, DerefMut)]
pub struct SaveFilter(SceneFilter);

impl FromWorld for SaveFilter {
    fn from_world(world: &mut World) -> Self {
        Self(
            SceneFilter::deny_all()
                .allow::<Name>() // NOCOMMIT temporary
        )
    }
}



pub fn NOCOMMIT_test_save_data() {
    let mut m = SaveData::default();
    let n = &m;
    let event = SaveEvent::AddMetadata("foo", "bar".to_string());
    n.send.send(event).unwrap();

    let o = m.recv.get_mut().unwrap();
    // processs
}
