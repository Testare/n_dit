use getset::{CopyGetters, Getters};
use serde::{Deserialize, Serialize};

use super::item::Item;
use crate::prelude::*;

#[derive(Debug, Default)]
pub struct ShopPlugin;

impl Plugin for ShopPlugin {
    fn build(&self, _app: &mut App) {}
}

#[derive(Component, Debug, Deref, DerefMut)]
pub struct InShop(pub Entity);

#[derive(Clone, Component, Debug, Default, Deserialize, Hash, PartialEq, Reflect, Serialize)]
#[reflect(Deserialize, Serialize)]
pub struct ShopId(pub SetId);

#[derive(Component, Debug, Default, Deref, DerefMut)]
pub struct ShopInventory(pub Vec<ShopListing>);

#[derive(Debug, Getters, CopyGetters)]
pub struct ShopListing {
    #[getset(get = "pub")]
    item: Item,
    #[getset(get_copy = "pub")]
    price: u32,
}

impl ShopListing {
    pub fn new(price: u32, item: Item) -> Self {
        ShopListing { item, price }
    }
}
