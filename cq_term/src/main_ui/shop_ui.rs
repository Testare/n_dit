use bevy::hierarchy::BuildWorldChildren;
use game_core::card::CardDefinition;
use game_core::common::daddy::Daddy;
use game_core::player::{ForPlayer, Player};
use game_core::shop::{InShop, ShopId, ShopInventory};
use getset::CopyGetters;

use crate::base_ui::context_menu::{ContextAction, ContextActions};
use crate::base_ui::ButtonUiBundle;
use crate::configuration::DrawConfiguration;
use crate::layout::VisibilityTty;
use crate::prelude::*;

#[derive(Debug)]
pub struct ShopUiPlugin;

impl Plugin for ShopUiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ShopUiContextActions>()
            .add_systems(Update, sys_open_shop_ui);
    }
}

#[derive(CopyGetters, Debug, Reflect, Resource)]
#[get_copy = "pub"]
pub struct ShopUiContextActions {
    buy_item: Entity,
    finish_shopping: Entity,
    select_item: Entity,
}

impl FromWorld for ShopUiContextActions {
    fn from_world(world: &mut World) -> Self {
        world.init_resource::<Daddy<ShopUiContextActions>>();
        let buy_item = world
            .spawn((
                Name::new("Buy item CA"),
                ContextAction::new("Buy item", |id, world| {
                    log::debug!("TODO Buy item {id:?}!");
                }),
            ))
            .id();
        let finish_shopping = world
            .spawn((
                Name::new("Finish shopping CA"),
                ContextAction::new("Finish shopping", |id, world| {
                    log::debug!("TODO Finish shopping {id:?}!");
                }),
            ))
            .id();
        let select_item = world
            .spawn((
                Name::new("Select item CA"),
                ContextAction::new("Select item", |id, world| {
                    log::debug!("TODO Select item {id:?}!");
                }),
            ))
            .id();
        world
            .entity_mut(
                *world
                    .get_resource::<Daddy<ShopUiContextActions>>()
                    .expect("daddy should've just been initialized")
                    .deref(),
            )
            .add_child(select_item)
            .add_child(buy_item)
            .add_child(finish_shopping);
        Self {
            buy_item,
            finish_shopping,
            select_item,
        }
    }
}

pub fn sys_open_shop_ui(
    mut commands: Commands,
    ast_card: Res<Assets<CardDefinition>>,
    res_shop_ui_ca: Res<ShopUiContextActions>,
    res_draw_config: Res<DrawConfiguration>,
    q_player_entering_shop: Query<(Entity, &InShop), (With<Player>, Added<InShop>)>,
    q_shop_listing_ui: Query<(&ForPlayer, Entity), With<ShopListingUi>>,
    q_shop: Query<AsDeref<ShopInventory>, With<ShopId>>,
    mut q_shop_ui: Query<(&ForPlayer, AsDerefMut<VisibilityTty>), With<ShopUi>>,
) {
    // TODO react if shop inventory changes
    for (player_id, &InShop(shop_id)) in q_player_entering_shop.iter() {
        // TODO display inventory as well for comparison
        // This means card selection and description stuff from node
        log::debug!("TODO (player [{player_id:?}] entered shop [{shop_id:?}])");
        if let Some((_, mut is_visible)) = q_shop_ui
            .iter_mut()
            .find(|(&ForPlayer(for_player), _)| for_player == player_id)
        {
            is_visible.set_if_neq(true);
        };

        q_shop_listing_ui
            .iter()
            .find(|(&ForPlayer(for_player), _)| for_player == player_id)
            .and_then(|(_, ui_id)| {
                let shop_inv = q_shop.get(shop_id).ok()?;

                commands.entity(ui_id).with_children(|listing_ui| {
                    for (i, listing) in shop_inv.iter().enumerate() {
                        // TODO Don't use ButtonUiBundle as shortcut, use custom render
                        // system
                        listing_ui.spawn((
                            ShopListingItemUi(i),
                            ButtonUiBundle::new(
                                format!(
                                    "{} - ${}",
                                    listing.item().name(&ast_card),
                                    listing.price()
                                ),
                                res_draw_config.color_scheme().shop_ui_listing_item(),
                            ),
                            ContextActions::new(
                                player_id, /* Shop UI ID? */
                                vec![res_shop_ui_ca.select_item(), res_shop_ui_ca.buy_item()],
                            ),
                        ));
                    }
                });
                Some(())
            });
    }
}

#[derive(Component, Debug)]
pub struct ShopUi;

#[derive(Component, Debug)]
pub struct ShopListingUi;

#[derive(Component, Debug)]
pub struct ShopListingItemUi(usize);

#[derive(Component, Debug)]
pub struct ItemDetailsUi;

#[derive(Component, Debug)]
pub struct ShopUiBuyButton;

#[derive(Component, Debug)]
pub struct ShopUiFinishShoppingButton;
