use std::fmt::Debug;
use std::path::PathBuf;

use bevy::asset::{AssetLoader, FileAssetIo, LoadedAsset};
use bevy::prelude::{AssetEvent, HandleUntyped};
use bevy::reflect::{TypePath, TypeUuid};
use glob::glob;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use crate::prelude::*;

pub trait Registry: Sync + Send + 'static {
    const REGISTRY_NAME: &'static str;
    type Value: DeserializeOwned + Sync + Send + 'static;
}

#[derive(Debug, Resource)]
pub struct Reg<R: Registry> {
    values: HashMap<String, (i32, R::Value)>,
}

impl<R: Registry> Reg<R> {
    pub fn get(&self, key: &str) -> Option<&R::Value> {
        self.values.get(key).map(|(_, v)| v)
    }

    fn add(&mut self, key: String, priority: i32, value: R::Value) {
        if self.values.contains_key(&key) {
            let (current_priority, current_value) = self.values.get_mut(&key).unwrap();
            if priority >= *current_priority {
                *current_priority = priority;
                *current_value = value;
            }
        } else {
            self.values.insert(key, (priority, value));
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
            .add_systems(PreUpdate, sys_consume_registry_file::<R>);
    }
}

#[derive(Serialize, Deserialize, TypeUuid, TypePath)]
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

#[derive(Debug)]
pub struct RegistryPlugin;

impl Plugin for RegistryPlugin {
    fn build(&self, app: &mut App) {
        app.add_asset_loader(RegistryTomlAssetLoader)
            .add_asset::<RegistryTomlFile>()
            .init_resource::<Registries>();
    }
}

#[derive(Resource)]
struct Registries(Vec<HandleUntyped>);

impl FromWorld for Registries {
    fn from_world(world: &mut World) -> Self {
        let handles = world
            .get_resource::<AssetServer>()
            .and_then(|asset_server| {
                let asset_io = asset_server.asset_io().downcast_ref::<FileAssetIo>()?;
                let path = asset_io.root_path().to_string_lossy().into_owned();
                // let mut json_path = path.clone();
                let mut toml_path = path;
                toml_path.push_str("/**/*.reg.toml");
                let paths = glob(toml_path.as_str())
                    .ok()?
                    .filter_map(|path| {
                        let path = path.ok()?;
                        log::info!("Found registry file {}", path.to_string_lossy());
                        Some(asset_server.load_untyped(path.to_str()?))
                    })
                    .collect();
                Some(paths)
            })
            .expect("should be able to load registry files");
        Registries(handles)
    }
}

struct RegistryTomlAssetLoader;

impl AssetLoader for RegistryTomlAssetLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut bevy::asset::LoadContext,
    ) -> bevy::utils::BoxedFuture<'a, Result<(), bevy::asset::Error>> {
        Box::pin(async move {
            let value_str = std::str::from_utf8(bytes)?;
            let mut registry_file = toml::from_str::<RegistryTomlFile>(value_str)?;
            registry_file.source_file = load_context.path().to_path_buf();
            load_context.set_default_asset(LoadedAsset::new(registry_file));
            Ok(())
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
) {
    for event in evr_asset.into_iter() {
        match event {
            AssetEvent::Created { handle } | AssetEvent::Modified { handle } => {
                let reg_file = ast_reg_files.get(handle).unwrap();
                if reg_file.registry() != R::REGISTRY_NAME || reg_file.values().is_empty() {
                    continue;
                }
                let reg_file = ast_reg_files.get_mut(handle).unwrap();
                let priority = reg_file.priority;
                for (key, value) in reg_file.values_mut().drain() {
                    match value.try_into::<R::Value>() {
                        Ok(value) => {
                            reg.add(key.clone(), priority, value);
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
