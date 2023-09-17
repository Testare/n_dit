use std::borrow::Cow;

use bevy::asset::{AssetLoader, AssetPath, LoadedAsset};
use bevy::reflect::TypeUuid;
use getset::{CopyGetters, Getters};
use serde::{Deserialize, Serialize};

use super::{ActionEffect, ActionRange, ActionTarget, Prerequisite, RangeShape};
use crate::prelude::*;

pub const NO_OP_ACTION_ID: Cow<'static, str> = Cow::Borrowed("No action");

// Will rename to differentiate the serialized definition from the in-game one
// Note: Deck will have to have nicknames for cards with metadata
#[derive(Debug, Deserialize, Serialize)]
pub struct CardAssetDef {
    #[serde(default)]
    actions: Vec<String>,
    short_name: Option<String>,
    description: String,
    display: String,
    display_id: Option<String>,
    max_size: u32,
    #[serde(default)]
    prevent_no_op: bool,
    speed: u32,
    #[serde(default)]
    tags: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ActionAssetDef {
    effects: Vec<ActionEffect>,
    #[serde(default)]
    prereqs: Vec<Prerequisite>,
    range: ActionRangeRepr,
    #[serde(default)]
    self_effects: Vec<ActionEffect>,
    #[serde(default)]
    tags: Vec<String>,
    target: ActionTarget,
    description: Option<String>, // TODO NOT OPTIONAL
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

#[derive(Clone, Debug, Getters, Reflect, TypeUuid)]
#[uuid = "fc3bb5f8-59f7-4e1e-8ea1-25d736483b6f"]
pub struct ActionDefinition {
    pub(crate) range: Option<ActionRange>,
    pub(crate) id: String,
    #[getset(get = "pub")]
    pub(crate) effects: Vec<ActionEffect>,
    #[getset(get = "pub")]
    pub(crate) self_effects: Vec<ActionEffect>,
    #[getset(get = "pub")]
    pub(crate) target: ActionTarget,
    #[getset(get = "pub")]
    pub(crate) tags: Vec<String>,
    #[getset(get = "pub")]
    pub(crate) prereqs: Vec<Prerequisite>,
    pub(crate) description: String,
}

#[derive(CopyGetters, Clone, Debug, Getters, Hash, PartialEq, Reflect, TypeUuid)]
#[uuid = "e8d74f73-96cf-4916-84c5-9041fa10c4ed"]
pub struct CardDefinition {
    id: String,
    #[getset(get = "pub")]
    short_name: String,
    #[getset(get = "pub")]
    actions: Vec<Handle<ActionDefinition>>,
    description: String,
    #[get_copy = "pub"]
    max_size: u32,
    #[get_copy = "pub"]
    movement_speed: u32,
    #[get_copy = "pub"]
    prevent_no_op: bool,
    #[getset(get = "pub")]
    display_id: String,
    #[getset(get = "pub")]
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
                    effects,
                    prereqs,
                    range,
                    self_effects,
                    tags,
                    target,
                    description,
                } = def;
                let def = ActionDefinition {
                    id: id.clone(),
                    range: Some(range.into()),
                    tags,
                    effects,
                    target,
                    self_effects,
                    prereqs,
                    description: description.unwrap_or("Description not found".into()),
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
                let mut action_assets = Vec::new();
                let mut actions = Vec::new();
                for action_path in def.actions.into_iter() {
                    actions.push(load_context.get_handle(action_path.as_str()));
                    if Some('#') != action_path.chars().next() {
                        action_assets.push(AssetPath::from(action_path));
                    }
                }
                let def = CardDefinition {
                    id: id.clone(),
                    actions,
                    short_name: def.short_name.unwrap_or_else(|| id.clone()),
                    description: def.description,
                    max_size: def.max_size,
                    movement_speed: def.speed,
                    prevent_no_op: def.prevent_no_op,
                    display_id: def.display_id.unwrap_or(def.display),
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

impl Default for ActionDefinition {
    fn default() -> Self {
        ActionDefinition {
            id: NO_OP_ACTION_ID.into_owned(),
            range: None,
            effects: Vec::new(),
            self_effects: Vec::new(),
            target: ActionTarget::None,
            tags: Vec::new(),
            prereqs: Vec::new(),
            description: "End turn and do nothing".into(),
        }
    }
}
impl ActionDefinition {
    pub fn id(&self) -> &str {
        self.id.as_str()
    }

    pub fn id_cow(&self) -> Cow<'static, str> {
        Cow::Owned(self.id.clone())
    }

    pub fn description(&self) -> &str {
        &self.description.as_str()
    }

    pub fn range(&self) -> Option<ActionRange> {
        self.range
    }
}

impl CardDefinition {
    pub fn id(&self) -> &str {
        self.id.as_str()
    }

    pub fn id_cow(&self) -> Cow<'static, str> {
        Cow::Owned(self.id.clone())
    }

    pub fn description(&self) -> &str {
        &self.description.as_str()
    }
}
