use std::fmt::Debug;
use std::marker::PhantomData;
use std::path::PathBuf;

use bevy::asset::io::Reader;
use bevy::asset::{AssetLoader, LoadContext, LoadedUntypedAsset};
use bevy::prelude::AssetApp;
use bevy::reflect::{TypePath, TypeUuid};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::prelude::*;

#[derive(Debug)]
pub struct RegistryPlugin;

impl Plugin for RegistryPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<RegistryTomlFile>()
            .init_asset_loader::<RegistryTomlAssetLoader>()
            .init_resource::<Registries>();
    }
}

pub trait Registry: Sync + Send + 'static {
    const REGISTRY_NAME: &'static str;
    type Value: Debug + DeserializeOwned + Sync + Send + 'static;

    /// Can be used to detect if a value is different from the old value. If it
    /// returns false, the old key value will not be overriden.
    ///
    /// Defaults to always saying two values are different, can be overriden
    /// to an unequality check for a specific registry to enable change detection.
    /// e.g., if [`Self::Value`] implements [`PartialEq`], you can just do the following:
    ///
    /// ```nocompile
    /// fn detect_change( old_value: &Self::Value, new_value: &Self::Value) -> bool {
    ///    old_value != new_value
    /// }
    /// ```
    #[allow(unused)]
    fn detect_change(old_value: &Self::Value, new_value: &Self::Value) -> bool {
        true
    }

    /// Whether the registry should emit events when the registry is updated.
    /// Default is `false`.
    ///
    /// When the registry is modified, it compares the new values with the old
    /// using [`detect_change`], and if it returns true, will emit
    /// [`UpdatedRegistryKey`] event for the key.
    ///
    /// If you implement this to return true without changing [`detect_change`],
    /// when a registry is updated events will be emitted for all keys it
    /// defines. If you update [`detect_change`] to do an equality check
    /// between old value and new, it will only emit events for the specific
    /// keys that have new values.
    ///
    /// Default is false.
    fn emit_change_events() -> bool {
        false
    }
}

#[derive(Debug, Resource)]
pub struct Reg<R: Registry> {
    values: HashMap<String, (i32, R::Value)>,
}

impl<R: Registry> Reg<R> {
    pub fn get(&self, key: &str) -> Option<&R::Value> {
        self.values.get(key).map(|(_, v)| v)
    }

    fn add(
        &mut self,
        key: String,
        priority: i32,
        value: R::Value,
        evw_key_updates: &mut EventWriter<UpdatedRegistryKey<R>>,
    ) {
        if !self.values.contains_key(&key) {
            self.values.insert(key, (priority, value));
            return;
        }

        let (current_priority, current_value) = self.values.get_mut(&key).unwrap();
        if priority < *current_priority {
            return;
        }
        *current_priority = priority;

        if !<R as Registry>::detect_change(current_value, &value) {
            return;
        }
        *current_value = value;

        if R::emit_change_events() {
            evw_key_updates.send(UpdatedRegistryKey::new(key));
        }
    }
}

impl<R: Registry> Default for Reg<R> {
    fn default() -> Self {
        Self {
            values: HashMap::new(),
        }
    }
}

impl<R: Registry> Plugin for Reg<R> {
    fn build(&self, app: &mut App) {
        app.insert_resource(Self::default())
            .add_event::<UpdatedRegistryKey<R>>()
            .add_systems(PreUpdate, sys_consume_registry_file::<R>);
    }
}

#[derive(Asset, Serialize, Deserialize, TypeUuid, TypePath)]
#[uuid = "60c5b905-a8a2-4194-828e-bb1f62432b37"]
struct RegistryTomlFile {
    #[serde(skip)]
    source_file: PathBuf,
    #[serde(default)]
    priority: i32,
    registry: String,
    #[serde(default)]
    values: HashMap<String, toml::Value>,
}

impl RegistryTomlFile {
    fn registry(&self) -> &str {
        &self.registry
    }

    fn values(&self) -> &HashMap<String, toml::Value> {
        &self.values
    }
    fn values_mut(&mut self) -> &mut HashMap<String, toml::Value> {
        &mut self.values
    }
}

#[derive(Resource)]
struct Registries(Vec<Handle<LoadedUntypedAsset>>);

impl FromWorld for Registries {
    fn from_world(_: &mut World) -> Self {
        Registries(Vec::new())
    }
}

#[derive(Debug, Default)]
struct RegistryTomlAssetLoader;

#[derive(Debug, Error)]
pub enum RegistryLoadError {
    #[error(transparent)]
    VoldemortError(#[from] toml::de::Error),
    #[error(transparent)]
    IanError(#[from] std::io::Error),
}

impl AssetLoader for RegistryTomlAssetLoader {
    type Asset = RegistryTomlFile;
    type Settings = ();
    type Error = RegistryLoadError;

    fn load<'a>(
        &'a self,
        reader: &'a mut Reader,
        _: &Self::Settings,
        load_context: &'a mut LoadContext,
    ) -> bevy::utils::BoxedFuture<'a, Result<Self::Asset, Self::Error>> {
        Box::pin(async move {
            let mut value_str = String::new();
            reader.read_to_string(&mut value_str).await?;
            let mut registry_file = toml::from_str::<RegistryTomlFile>(value_str.as_str())?;
            registry_file.source_file = load_context.path().to_path_buf();
            Ok(registry_file)
        })
    }

    fn extensions(&self) -> &[&str] {
        &["reg.toml"]
    }
}

fn sys_consume_registry_file<R: Registry>(
    mut reg: ResMut<Reg<R>>,
    mut ast_reg_files: ResMut<Assets<RegistryTomlFile>>,
    mut evr_asset: EventReader<AssetEvent<RegistryTomlFile>>,
    mut evw_reg_change: EventWriter<UpdatedRegistryKey<R>>,
) {
    for event in evr_asset.read() {
        match event {
            AssetEvent::LoadedWithDependencies { id } | AssetEvent::Modified { id } => {
                let reg_file = ast_reg_files.get(*id).unwrap();
                if reg_file.registry() != R::REGISTRY_NAME || reg_file.values().is_empty() {
                    continue;
                }
                let reg_file = ast_reg_files.get_mut(*id).unwrap();
                let priority = reg_file.priority;
                for (key, value) in reg_file.values_mut().drain() {
                    match value.try_into::<R::Value>() {
                        Ok(value) => {
                            reg.add(key.clone(), priority, value, &mut evw_reg_change);
                            log::trace!("Registry [{}] loaded key [{}]", R::REGISTRY_NAME, key);
                        },
                        Err(e) => log::warn!(
                            "Error reading registry[{}] value[{}]: {:?}",
                            R::REGISTRY_NAME,
                            key,
                            e
                        ),
                    }
                }
            },
            _ => {},
        }
    }
}

#[derive(Debug, Deref, Event)]
pub struct UpdatedRegistryKey<R> {
    #[deref]
    key: String,
    phantom_data: PhantomData<R>,
}

impl<R> UpdatedRegistryKey<R> {
    fn new<S: Into<String>>(key: S) -> Self {
        UpdatedRegistryKey {
            key: key.into(),
            phantom_data: PhantomData::<R>,
        }
    }

    pub fn key(&self) -> &str {
        self.key.as_str()
    }
}
