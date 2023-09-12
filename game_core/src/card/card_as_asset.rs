use bevy::asset::{AssetLoader, AssetPath, HandleId, LoadedAsset};
use bevy::reflect::TypeUuid;
use getset::{CopyGetters, Getters};
use serde::{Deserialize, Serialize};

use crate::prelude::*;

// Will rename to differentiate the serialized definition from the in-game one
// Note: Deck will have to have nicknames for cards with metadata
#[derive(Debug, Deserialize, Serialize)]
pub struct CardAssetDef {
    actions: Vec<String>,
    description: String,
    display: String,
    max_size: usize,
    speed: usize,
    #[serde(default)]
    tags: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ActionAssetDef {
    range: usize,
}

#[derive(Debug, Reflect, TypeUuid)]
#[uuid = "fc3bb5f8-59f7-4e1e-8ea1-25d736483b6f"]
pub struct ActionDefinition {
    range: usize,
    id: String,
}

#[derive(CopyGetters, Clone, Debug, Getters, Hash, PartialEq, Reflect, TypeUuid)]
#[uuid = "e8d74f73-96cf-4916-84c5-9041fa10c4ed"]
pub struct CardDefinition {
    #[getset(get = "pub")]
    actions: Vec<Handle<ActionDefinition>>,
    description: String,
    max_size: usize,
    movement_speed: usize,
    display_id: String,
    tags: Vec<String>,
}

pub struct ActionAssetLoader;

impl AssetLoader for ActionAssetLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut bevy::asset::LoadContext,
    ) -> bevy::utils::BoxedFuture<'a, Result<(), bevy::asset::Error>> {
        Box::pin(async move {
            let asset_map: HashMap<String, ActionAssetDef> = serde_json::from_slice(bytes)?;
            for (id, def) in asset_map.into_iter() {
                let def = ActionDefinition {
                    id: id.clone(),
                    range: def.range,
                };
                load_context.set_labeled_asset(id.as_str(), LoadedAsset::new(def));
            }
            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["actions.json"]
    }
}
#[derive(Default)]
pub struct CardAssetLoader;

impl AssetLoader for CardAssetLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut bevy::asset::LoadContext,
    ) -> bevy::utils::BoxedFuture<'a, Result<(), bevy::asset::Error>> {
        Box::pin(async move {
            let asset_map: HashMap<String, CardAssetDef> = serde_json::from_slice(bytes)?;
            for (id, def) in asset_map.into_iter() {
                let (action_assets, actions) = def
                    .actions
                    .iter()
                    .map(|action_id| {
                        // TODO Better action ids and perhaps a better way to specify a label
                        let asset_path = format!("nightfall/program.actions.json#{}", action_id);
                        let handle = load_context.get_handle(asset_path.as_str());
                        (AssetPath::from(asset_path), handle)
                    })
                    .unzip();
                let def = CardDefinition {
                    actions,
                    description: def.description,
                    max_size: def.max_size,
                    movement_speed: def.speed,
                    display_id: def.display,
                    tags: def.tags,
                };

                load_context.set_labeled_asset(
                    id.as_str(),
                    LoadedAsset::new(def).with_dependencies(action_assets),
                );
            }

            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["cards.json"]
    }
}
