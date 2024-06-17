use std::borrow::Cow;

use bevy::asset::io::Reader;
use bevy::asset::{AssetLoader, LoadContext};
use getset::{CopyGetters, Getters};
use serde::{Deserialize, Serialize};

use super::{Action, ActionEffect, ActionRange, ActionTarget, Prerequisite, RangeShape};
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
    description: String,
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

#[derive(Asset, CopyGetters, Clone, Debug, Getters, Hash, PartialEq, Reflect)]
pub struct CardDefinition {
    id: String,
    #[getset(get = "pub")]
    short_name: String,
    #[getset(get = "pub")]
    actions: Vec<Handle<Action>>,
    description: String,
    #[get_copy = "pub"]
    max_size: u32,
    #[get_copy = "pub"]
    movement_speed: u32,
    #[get_copy = "pub"]
    prevent_no_op: bool,
    #[getset(get = "pub")]
    tags: Vec<String>,
}

#[derive(Default)]
pub struct ActionAssetLoader;

impl AssetLoader for ActionAssetLoader {
    type Asset = ();
    type Settings = ();
    type Error = std::io::Error;

    async fn load<'a>(
        &'a self,
        reader: &'a mut Reader<'_>,
        _: &'a Self::Settings,
        load_context: &'a mut LoadContext<'_>,
    ) -> Result<(), Self::Error> {
        let mut bytes = vec![];
        reader.read_to_end(&mut bytes).await?;
        let asset_map: HashMap<String, ActionAssetDef> = serde_json::from_slice(&bytes[..])?;
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
            let def = Action {
                id: id.clone(),
                range: Some(range.into()),
                tags,
                effects,
                target,
                self_effects,
                prereqs,
                description,
            };
            if let Err(err_msg) = validations::validate_action_effects_match_target(&def) {
                log::error!("{}", err_msg);
            } else {
                load_context.labeled_asset_scope(id, |_| def);
            }
        }
        Ok(())
    }

    fn extensions(&self) -> &[&str] {
        &["actions.json"]
    }
}

#[derive(Default)]
pub struct CardAssetLoader;

impl AssetLoader for CardAssetLoader {
    type Asset = ();
    type Settings = ();
    type Error = std::io::Error;

    async fn load<'a>(
        &'a self,
        reader: &'a mut Reader<'_>,
        _: &'a Self::Settings,
        load_context: &'a mut bevy::asset::LoadContext<'_>,
    ) -> Result<(), Self::Error> {
        let mut bytes = vec![];
        reader.read_to_end(&mut bytes).await?;
        let asset_map: HashMap<String, CardAssetDef> = serde_json::from_slice(&bytes[..])?;
        for (id, def) in asset_map.into_iter() {
            load_context.labeled_asset_scope(id.clone(), |lc| {
                let mut actions = Vec::new();
                for action_path in def.actions.into_iter() {
                    actions.push(lc.load(action_path.to_string()));
                }
                CardDefinition {
                    id: id.clone(),
                    actions,
                    short_name: def.short_name.unwrap_or(id),
                    description: def.description,
                    max_size: def.max_size,
                    movement_speed: def.speed,
                    prevent_no_op: def.prevent_no_op,
                    tags: def.tags,
                }
            });
        }

        Ok(())
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

impl Default for Action {
    fn default() -> Self {
        Action {
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
impl Action {
    pub fn id(&self) -> &str {
        self.id.as_str()
    }

    pub fn id_cow(&self) -> Cow<'static, str> {
        Cow::Owned(self.id.clone())
    }

    pub fn description(&self) -> &str {
        self.description.as_str()
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
        self.description.as_str()
    }
}

mod validations {
    use super::Action;
    use crate::card::ActionTarget;

    pub fn validate_action_effects_match_target(action: &Action) -> Result<(), String> {
        let target = &action.target;
        for (i, effect) in action.effects().iter().enumerate() {
            if !effect.valid_target(target) {
                return Err(format!(
                    "Invalid action {} - Effect [{}] does not match target",
                    action.id(),
                    i
                ));
            }
        }
        for (i, effect) in action.self_effects().iter().enumerate() {
            if !effect.valid_target(&ActionTarget::Curios) {
                return Err(format!(
                    "Invalid action {} - Self-effect [{}] does not match target",
                    action.id(),
                    i
                ));
            }
        }
        Ok(())
    }
}
