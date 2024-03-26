use std::borrow::Cow;

use game_core::card::{BaseName, Card, CardQueryItem};
use game_core::registry::{Reg, Registry, UpdatedRegistryKey};
use serde::{Deserialize, Serialize};

use crate::prelude::*;

#[derive(Debug)]
pub struct CardUiPlugin;

impl Plugin for CardUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(Reg::<ShortName>::default())
            .add_systems(Update, (sys_add_short_name_to_cards, sys_update_short_name));
    }
}

#[derive(Debug, Clone, Component, Serialize, Deref, Deserialize, PartialEq)]
#[serde(transparent)]
#[component(storage = "SparseSet")]
pub struct ShortName(pub String);

impl Registry for ShortName {
    const REGISTRY_NAME: &'static str = "term:short_names";
    type Value = ShortName;

    fn detect_change(old_value: &Self::Value, new_value: &Self::Value) -> bool {
        old_value != new_value
    }
    fn emit_change_events() -> bool {
        true
    }
}

impl ShortName {
    pub fn nickname_or_short_name(
        short_name: Option<&ShortName>,
        card_query: CardQueryItem,
    ) -> Cow<'static, str> {
        Cow::Owned(
            card_query
                .nickname
                .or(short_name.map(|short_name| &**short_name))
                .unwrap_or(card_query.base_name)
                .clone(),
        )
    }
}

pub fn sys_add_short_name_to_cards(
    mut commands: Commands,
    reg_short_names: Res<Reg<ShortName>>,
    q_card: Query<(Entity, AsDeref<BaseName>), Added<Card>>,
) {
    for (id, base_name) in q_card.iter() {
        if let Some(short_name) = reg_short_names.get(base_name.as_str()) {
            commands.entity(id).insert(short_name.clone());
        }
    }
}

pub fn sys_update_short_name(
    mut commands: Commands,
    mut evr_reg_update: EventReader<UpdatedRegistryKey<ShortName>>,
    reg_short_names: Res<Reg<ShortName>>,
    q_card: Query<(Entity, AsDeref<BaseName>), With<Card>>,
) {
    for updated_key in evr_reg_update.read() {
        for (id, base_name) in q_card.iter() {
            if *base_name == **updated_key {
                if let Some(short_name) = reg_short_names.get(base_name.as_str()) {
                    commands.entity(id).insert(short_name.clone());
                }
            }
        }
    }
}
