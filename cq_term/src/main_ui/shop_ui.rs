use crossterm::style::{ContentStyle, Stylize};
use game_core::card::CardDefinition;
use game_core::player::{ForPlayer, Player};
use game_core::shop::{InShop, ShopId, ShopInventory};

use crate::base_ui::ButtonUiBundle;
use crate::layout::VisibilityTty;
use crate::prelude::*;

#[derive(Debug)]
pub struct ShopUiPlugin;

impl Plugin for ShopUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, sys_open_shop_ui);
    }
}

pub fn sys_open_shop_ui(
    mut commands: Commands,
    ast_card: Res<Assets<CardDefinition>>,
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
                                ContentStyle::new().cyan().on_dark_blue(),
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
