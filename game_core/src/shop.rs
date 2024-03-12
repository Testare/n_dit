use crate::prelude::*;
use super::item::Item;

#[derive(Debug, Default)]
pub struct ShopPlugin;

impl Plugin for ShopPlugin {
    fn build(&self, app: &mut App) {
        
    }
}

#[derive(Component, Debug, Default)]
pub struct ShopInventory(Vec<ShopListing>);

#[derive(Debug)]
pub struct ShopListing {
    item: Item,
    price: u32,
}