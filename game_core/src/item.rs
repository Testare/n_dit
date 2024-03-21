use std::borrow::Cow;

use getset::CopyGetters;

use crate::card::CardDefinition;
use crate::prelude::*;

pub const MAX_MON: u32 = 100_000_000;

#[derive(Debug, Default)]
pub struct ItemPlugin;

impl Plugin for ItemPlugin {
    fn build(&self, _app: &mut App) {}
}

#[derive(Component, CopyGetters, Debug, Default, Reflect)]
#[get_copy = "pub"]
pub struct Wallet {
    mon: u32,
}

impl Wallet {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_mon(mut self, mon: u32) -> Self {
        self.mon = mon;
        self
    }

    pub fn increase_mon(&mut self, mon: u32) {
        self.mon = self.mon.saturating_add(mon).min(MAX_MON);
    }

    pub fn decrease_mon(&mut self, mon: u32) {
        self.mon = self.mon.saturating_sub(mon);
    }
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
