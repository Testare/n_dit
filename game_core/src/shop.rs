use getset::{CopyGetters, Getters};
use serde::{Deserialize, Serialize};

use super::item::Item;
use crate::op::{Op, OpImplResult, OpPlugin, OpRegistrar};
use crate::prelude::*;

#[derive(Debug, Default)]
pub struct ShopPlugin;

impl Plugin for ShopPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(OpPlugin::<ShopOp>::default());
    }
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

#[derive(Clone, Debug, Reflect)]
pub enum ShopOp {
    BuyItem(usize),
    Enter(ShopId),
    Leave,
}

impl Op for ShopOp {
    fn register_systems(mut registrar: OpRegistrar<Self>) {
        // TODO TODONEXT
        registrar
            .register_op(opsys_debug)
            .register_op(opsys_debug)
            .register_op(opsys_debug);
    }

    fn system_index(&self) -> usize {
        match self {
            Self::BuyItem(_) => 0,
            Self::Enter(_) => 1,
            Self::Leave => 2,
        }
    }
}

pub fn opsys_debug(In((player_id, shop_op)): In<(Entity, ShopOp)>) -> OpImplResult {
    log::debug!("TODO TODONEXT {shop_op:?} for {player_id:?}");
    Ok(default())
}
