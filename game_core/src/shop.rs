use std::str::FromStr;

use bevy::ecs::query::Has;
use getset::{CopyGetters, Getters};
use serde::{Deserialize, Serialize};

use super::item::Item;
use crate::card::CardDefinition;
use crate::item::{ItemOp, Wallet};
use crate::op::{CoreOps, Op, OpError, OpErrorUtils, OpImplResult, OpPlugin, OpRegistrar};
use crate::player::Player;
use crate::prelude::*;

pub mod key {
    use typed_key::{typed_key, Key};

    pub const ITEM_NAME: Key<String> = typed_key!("item_name");
}

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

impl From<SetId> for ShopId {
    fn from(value: SetId) -> Self {
        Self(value)
    }
}

impl FromStr for ShopId {
    type Err = <SetId as FromStr>::Err;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(SetId::from_str(s)?))
    }
}

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
    BuyItem(usize), // TODO, perhaps also allow buying by name?
    Enter(ShopId),
    Leave,
}

impl Op for ShopOp {
    fn register_systems(mut registrar: OpRegistrar<Self>) {
        registrar
            .register_op(opsys_buy_item)
            .register_op(opsys_enter)
            .register_op(opsys_leave);
    }

    fn system_index(&self) -> usize {
        match self {
            Self::BuyItem(_) => 0,
            Self::Enter(_) => 1,
            Self::Leave => 2,
        }
    }
}

pub fn opsys_buy_item(
    In((player_id, shop_op)): In<(Entity, ShopOp)>,
    ast_card_def: Res<Assets<CardDefinition>>,
    mut res_core_ops: ResMut<CoreOps>,
    mut q_player: Query<(&InShop, &mut Wallet), With<Player>>,
    q_shop: Query<&ShopInventory, With<ShopId>>,
) -> OpImplResult {
    // TODO Add item to inventory
    if let ShopOp::BuyItem(item_idx) = shop_op {
        let (&InShop(shop_id), mut wallet) = q_player.get_mut(player_id).invalid()?;
        let mut metadata = Metadata::default();
        let shop_inventory = q_shop.get(shop_id).invalid()?;
        let listing = shop_inventory
            .get(item_idx)
            .ok_or("No item listed for that index")?;
        let can_pay = wallet.try_spend(listing.price());
        if !can_pay {
            Err("Cannot afford that item".invalid())?;
        }
        let name = listing.item().name(&ast_card_def);
        metadata
            .put(key::ITEM_NAME, name.to_string())
            .expect("it would be crazy if you couldn't deserialize a string");

        res_core_ops.request(
            player_id,
            ItemOp::AddItem {
                item: listing.item().clone(),
                refund: listing.price(),
            },
        );
        log::debug!(
            "Player bought [{:?}] Remaining Mon [{wallet:?}]",
            listing.item()
        );
        Ok(metadata)
    } else {
        Err(OpError::MismatchedOpSystem)
    }
}

pub fn opsys_enter(
    In((player_id, shop_op)): In<(Entity, ShopOp)>,
    mut commands: Commands,
    q_shop: Query<(Entity, &ShopId)>,
    q_player: Query<Has<InShop>, With<Player>>,
) -> OpImplResult {
    if let ShopOp::Enter(shop_sid) = shop_op {
        let already_in_shop = q_player.get(player_id).invalid()?;
        if already_in_shop {
            Err("Player already in a shop".invalid())?;
        }
        let shop_id = q_shop
            .iter()
            .find_map(|(id, i_shop_sid)| (shop_sid == *i_shop_sid).then_some(id))
            .ok_or("No shop matching that shop id".invalid())?;
        commands.entity(player_id).insert(InShop(shop_id));
        Ok(default())
    } else {
        Err(OpError::MismatchedOpSystem)
    }
}
pub fn opsys_leave(
    In((player_id, shop_op)): In<(Entity, ShopOp)>,
    mut commands: Commands,
    q_player: Query<Has<InShop>, With<Player>>,
) -> OpImplResult {
    if let ShopOp::Leave = shop_op {
        let in_shop = q_player.get(player_id).invalid()?;
        if !in_shop {
            Err("Player not in a shop".invalid())?;
        }
        commands.entity(player_id).remove::<InShop>();
        Ok(default())
    } else {
        Err(OpError::MismatchedOpSystem)
    }
}
