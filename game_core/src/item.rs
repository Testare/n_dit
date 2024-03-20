use std::borrow::Cow;

use crate::card::CardDefinition;
use crate::prelude::*;

#[derive(Debug, Default)]
pub struct ItemPlugin;

impl Plugin for ItemPlugin {
    fn build(&self, _app: &mut App) {}
}

#[derive(Debug)]
pub enum Item {
    Card(Handle<CardDefinition>),
    Mon(u32), // Others?
}

impl Item {
    pub fn name(&self, cards: &Assets<CardDefinition>) -> Cow<str> {
        match self {
            Self::Mon(_) => Cow::from("Mon"),
            Self::Card(handle) => cards
                .get(handle)
                .map(|card_def| Cow::Owned(card_def.id().to_owned()))
                .unwrap_or_else(|| {
                    log::error!("Unable to retreive name for card {handle:?}");
                    Cow::from("???")
                }),
        }
    }
}
