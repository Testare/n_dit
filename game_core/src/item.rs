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
