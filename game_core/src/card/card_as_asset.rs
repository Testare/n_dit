use bevy::asset::{AssetLoader, AssetPath, LoadedAsset};
use bevy::reflect::TypeUuid;
use getset::{CopyGetters, Getters};
use serde::{Deserialize, Serialize};

use super::{ActionEffect, ActionRange, ActionTarget, RangeShape};
use crate::prelude::*;

// Will rename to differentiate the serialized definition from the in-game one
// Note: Deck will have to have nicknames for cards with metadata
#[derive(Debug, Deserialize, Serialize)]
pub struct CardAssetDef {
    #[serde(default)]
    actions: Vec<String>,
    short_name: Option<String>,
    description: String,
    display: String,
    max_size: usize,
    speed: usize,
    #[serde(default)]
    tags: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ActionAssetDef {
    effects: Vec<ActionEffect>,
    range: ActionRangeRepr,
    #[serde(default)]
    self_effects: Vec<ActionEffect>,
    #[serde(default)]
    tags: Vec<String>,
    target: ActionTarget,
    // prereqs
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum ActionRangeRepr {
    Basic(u32),
    Complex {
        headless: Option<bool>,
        max_range: u32,
        shape: Option<RangeShape>,
        min_range: Option<u32>,
    },
}

#[derive(Debug, Reflect, TypeUuid)]
#[uuid = "fc3bb5f8-59f7-4e1e-8ea1-25d736483b6f"]
pub struct ActionDefinition {
    range: ActionRange,
    id: String,
    effect: ActionEffect,
    target: ActionTarget,
    tags: Vec<String>,
}

#[derive(CopyGetters, Clone, Debug, Getters, Hash, PartialEq, Reflect, TypeUuid)]
#[uuid = "e8d74f73-96cf-4916-84c5-9041fa10c4ed"]
pub struct CardDefinition {
    id: String,
    short_name: String,
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
                let ActionAssetDef {
                    tags,
                    range,
                    effects,
                    target,
                    self_effects,
                } = def;
                let def = ActionDefinition {
                    id: id.clone(),
                    range: range.into(),
                    tags,
                    effect: effects[0].clone(),
                    target,
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
                    .into_iter()
                    .map(|asset_path| {
                        let handle = load_context.get_handle(asset_path.as_str());
                        (AssetPath::from(asset_path), handle)
                    })
                    .unzip();
                let def = CardDefinition {
                    id: id.clone(),
                    actions,
                    short_name: def.short_name.unwrap_or_else(|| id.clone()),
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

impl ActionRangeRepr {
    fn into(self) -> ActionRange {
        match self {
            ActionRangeRepr::Basic(max_range) => ActionRange::new(max_range),
            ActionRangeRepr::Complex {
                headless,
                max_range,
                shape,
                min_range,
            } => ActionRange::new(max_range)
                .shaped(shape.unwrap_or_default())
                .min_range(min_range.unwrap_or_default())
                .headless(headless.unwrap_or_default()),
        }
    }
}

impl From<ActionRange> for ActionRangeRepr {
    fn from(value: ActionRange) -> Self {
        ActionRangeRepr::Complex {
            headless: Some(value.is_headless()),
            max_range: value.max_range(),
            shape: Some(value.shape()),
            min_range: Some(value.minimum_range()),
        }
    }
}
