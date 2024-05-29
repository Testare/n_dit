use bevy::hierarchy::{BuildWorldChildren, DespawnRecursiveExt};
use charmi::{CharacterMapImage, CharmieAnimation};
use crossterm::style::{Color, ContentStyle, Stylize};
use game_core::card::CardDefinition;
use game_core::common::daddy::Daddy;
use game_core::op::OpResult;
use game_core::player::{ForPlayer, Player};
use game_core::shop::{self, InShop, ShopId, ShopInventory, ShopOp};
use game_core::NDitCoreSet;
use getset::CopyGetters;

use super::UiOps;
use crate::animation::AnimationPlayer;
use crate::base_ui::context_menu::{ContextAction, ContextActions};
use crate::base_ui::{ButtonUiBundle, FlexibleTextUi, FlexibleTextUiMultiline};
use crate::configuration::DrawConfiguration;
use crate::layout::VisibilityTty;
use crate::linkage;
use crate::prelude::*;

#[derive(Debug)]
pub struct ShopUiPlugin;

impl Plugin for ShopUiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ShopUiContextActions>().add_systems(
            Update,
            (
                (sys_open_shop_ui, sys_leave_shop_ui, sys_update_item_details)
                    .in_set(NDitCoreSet::PostProcessUiOps),
                sys_buy_notification_ui.in_set(NDitCoreSet::PostProcessCommands),
            ),
        );
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
        let daddy = *world
            .get_resource::<Daddy<ShopUiContextActions>>()
            .expect("daddy should've just been initialized")
            .deref();
        let buy_item_sys = world.register_system(
            |In(id): In<Entity>,
             mut res_ui_ops: ResMut<UiOps>,
             q_shop_listing_item_ui: Query<(&ForPlayer, &ShopListingItemUi)>,
             q_buy_button: Query<&ForPlayer, With<ShopUiBuyButton>>,
             q_shop_ui: Query<(&ForPlayer, &ShopUiSelectedItem), With<ShopUi>>| {
                let player_id_and_buy_index = q_shop_listing_item_ui
                    .get(id)
                    .ok()
                    .map(|(&ForPlayer(player_id), &ShopListingItemUi(item_idx))| {
                        (player_id, item_idx)
                    })
                    .or_else(|| {
                        q_buy_button.get(id).ok().and_then(|&ForPlayer(player_id)| {
                            let item_idx = q_shop_ui.iter().find_map(
                                |(&ForPlayer(for_player), &ShopUiSelectedItem(listing_item_id))| {
                                    if for_player == player_id {
                                        if let Some((_, &ShopListingItemUi(item_idx))) /*(_, ShopListingItemUi(item_idx))*/ = listing_item_id.and_then(|listing_item_id|q_shop_listing_item_ui.get(listing_item_id).ok()) {
                                            Some(Some(item_idx))
                                        } else {
                                            Some(None) // We found the right ui, but there is no item selected
                                        }
                                    } else {
                                        None
                                    }
                                },
                            )??;
                            Some((player_id, item_idx))
                        })
                    });

                if let Some((player_id, item_idx)) = player_id_and_buy_index {
                    res_ui_ops.request(player_id, ShopOp::BuyItem(item_idx));
                } else {
                    log::warn!("Trying to buy item, but no item selected")
                }
            },
        );
        let select_item_sys =
            world.register_system(
                |In(id): In<Entity>,
                 res_draw_config: Res<DrawConfiguration>,
                 mut q_shop_listing_ui: Query<
                    (&ForPlayer, &mut FlexibleTextUi),
                    With<ShopListingItemUi>,
                >,
                 mut q_shop_ui: Query<
                    (&ForPlayer, AsDerefMut<ShopUiSelectedItem>),
                    With<ShopUi>,
                >| {
                    let old_selected_id = get_assert_mut!(
                        id,
                        q_shop_listing_ui,
                        |(&ForPlayer(player_id), mut flexible_text_ui)| {
                            let (_, mut shop_ui_selected_item) = q_shop_ui
                                .iter_mut()
                                .find(|(&ForPlayer(for_player), _)| player_id == for_player)?;
                            let old_selected_id = *shop_ui_selected_item;
                            if old_selected_id != Some(id) {
                                *shop_ui_selected_item = Some(id);
                                flexible_text_ui.style = res_draw_config
                                    .color_scheme()
                                    .shop_ui_listing_item_selected();
                                old_selected_id
                            } else {
                                None // They match, take no action
                            }
                        }
                    );
                    // Reset colors on old selected item
                    if let Some((_, mut flexible_text)) =
                        old_selected_id.and_then(|id| get_assert_mut!(id, q_shop_listing_ui))
                    {
                        flexible_text.style = res_draw_config.color_scheme().shop_ui_listing_item();
                    }
                },
            );
        let buy_item = world
            .spawn((
                Name::new("Buy item CA"),
                ContextAction::from_system_id("Buy item", buy_item_sys),
            ))
            .set_parent(daddy)
            .id();
        let finish_shopping = world
            .spawn((
                Name::new("Finish shopping CA"),
                linkage::base_ui_game_core::context_action_from_op::<UiOps, _>(
                    "Finish shopping",
                    ShopOp::Leave,
                ),
            ))
            .set_parent(daddy)
            .id();
        let select_item = world
            .spawn((
                Name::new("Select item CA"),
                ContextAction::from_system_id("Select item", select_item_sys),
            ))
            .set_parent(daddy)
            .id();
        Self {
            buy_item,
            finish_shopping,
            select_item,
        }
    }
}

#[derive(Component, Debug, Default)]
pub struct ShopNotification;

#[derive(Component, Debug)]
pub struct ShopUi;

#[derive(Component, Debug, Default, Deref, DerefMut)]
pub struct ShopUiSelectedItem(Option<Entity>);

#[derive(Component, Debug)]
pub struct ShopListingUi;

#[derive(Component, Debug)]
pub struct ShopListingItemUi(usize);

#[derive(Component, Debug)]
pub struct ItemDetailsUi;

#[derive(Component, Debug)]
pub struct ItemDetailsUiDescription;

#[derive(Component, Debug)]
pub struct ShopUiBuyButton;

#[derive(Component, Debug)]
pub struct ShopUiFinishShoppingButton;

pub fn sys_open_shop_ui(
    mut commands: Commands,
    ast_card: Res<Assets<CardDefinition>>,
    res_shop_ui_ca: Res<ShopUiContextActions>,
    res_draw_config: Res<DrawConfiguration>,
    mut evr_shop_op: EventReader<OpResult<ShopOp>>,
    q_player_entering_shop: Query<&InShop, With<Player>>,
    q_shop_listing_ui: Query<(&ForPlayer, Entity), With<ShopListingUi>>,
    q_shop: Query<AsDeref<ShopInventory>, With<ShopId>>,
    mut q_shop_ui: Query<(&ForPlayer, AsDerefMut<VisibilityTty>), With<ShopUi>>,
) {
    // TODO react if shop inventory changes
    for shop_op_result in evr_shop_op.read() {
        if !shop_op_result.result().is_ok() || !matches!(shop_op_result.op(), ShopOp::Enter(_)) {
            continue; // Not relevant
        }
        let player_id = shop_op_result.source();
        // If they entered and left same frame, they might not be there
        if let Ok(&InShop(shop_id)) = q_player_entering_shop.get(player_id) {
            // TODO display inventory as well for comparison
            // This means card selection and description stuff from node
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

                    commands
                        .entity(ui_id)
                        .despawn_descendants() // If any already exist
                        .with_children(|listing_ui| {
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
                                        &[res_shop_ui_ca.select_item(), res_shop_ui_ca.buy_item()],
                                    ),
                                    ForPlayer(player_id),
                                ));
                            }
                        });
                    Some(())
                });
        }
    }
}

fn sys_leave_shop_ui(
    mut commands: Commands,
    mut evr_shop_op: EventReader<OpResult<ShopOp>>,
    q_shop_listing_ui: Query<(&ForPlayer, Entity), With<ShopListingUi>>,
    q_player_not_in_shop: Query<(), (With<Player>, Without<InShop>)>,
    mut q_shop_ui: Query<
        (
            &ForPlayer,
            AsDerefMut<VisibilityTty>,
            AsDerefMut<ShopUiSelectedItem>,
        ),
        With<ShopUi>,
    >,
) {
    for shop_op_result in evr_shop_op.read() {
        if let (ShopOp::Leave, Ok(_)) = (shop_op_result.op(), shop_op_result.result()) {
            let player_id = shop_op_result.source();
            if !q_player_not_in_shop.contains(player_id) {
                // It's possible the user left the shop and entered another in
                // the same frame. In this case, we do nothing
                continue;
            }
            if let Some((_, mut is_visible, mut selected_item)) = q_shop_ui
                .iter_mut()
                .find(|(&ForPlayer(for_player), _, _)| for_player == player_id)
            {
                is_visible.set_if_neq(false);
                selected_item.set_if_neq(None);
            };
            // Not technically necessary since the game despawns children when a
            // player enters a shop, and the ShopUi is invisible, but it's good
            // to clean up after yourself.
            if let Some((_, ui_id)) = q_shop_listing_ui
                .iter()
                .find(|(&ForPlayer(for_player), _)| for_player == player_id)
            {
                commands.entity(ui_id).despawn_descendants();
            }
        }
    }
}

fn sys_update_item_details(
    ast_card_def: Res<Assets<CardDefinition>>,
    q_shop_ui: Query<
        (&ForPlayer, &ShopUiSelectedItem),
        (With<ShopUi>, Changed<ShopUiSelectedItem>),
    >,
    q_shop_listing: Query<&ShopListingItemUi>,
    q_player_entering_shop: Query<&InShop, With<Player>>,
    q_shop: Query<AsDeref<ShopInventory>, With<ShopId>>,
    mut q_shop_item_desc: Query<
        (
            &ForPlayer,
            AsDerefMut<VisibilityTty>,
            &mut FlexibleTextUiMultiline,
        ),
        With<ItemDetailsUiDescription>,
    >,
) {
    for (&ForPlayer(player_id), &ShopUiSelectedItem(selection)) in q_shop_ui.iter() {
        if let Some((_, mut visibility, mut flexible_text)) =
            ForPlayer::get_mut(&mut q_shop_item_desc, player_id)
        {
            let text_desc = selection.and_then(|selection_id|{ //try 
                let &ShopListingItemUi(selection_idx) = q_shop_listing.get(selection_id).ok()?;
                let &InShop(shop_id) = q_player_entering_shop.get(player_id).ok()?;
                let shop_inventory = q_shop.get(shop_id).ok()?;
                let listing = shop_inventory.get(selection_idx)?;
                Some(listing.item().description(&ast_card_def))
            });

            visibility.set_if_neq(text_desc.is_some());
            if let Some(text_desc) = text_desc {
                flexible_text.text = text_desc.into_owned();
            }
        }
    }
}

fn sys_buy_notification_ui(
    mut evr_shop_op: EventReader<OpResult<ShopOp>>,
    mut ast_animation: ResMut<Assets<CharmieAnimation>>,
    mut q_shop_notification: Query<(&ForPlayer, &mut AnimationPlayer), With<ShopNotification>>,
) {
    for shop_op_result in evr_shop_op.read() {
        if let (ShopOp::BuyItem(_), Ok(metadata)) = (shop_op_result.op(), shop_op_result.result()) {
            for (&ForPlayer(player_id), mut animation_player) in q_shop_notification.iter_mut() {
                if shop_op_result.source() != player_id {
                    continue;
                }
                // TODO Sound effect!
                if let Ok(item_name) = metadata.get_required(shop::key::ITEM_NAME) {
                    let animation = generate_buy_notification_animation(item_name.as_str());
                    let animation_handle = ast_animation.add(animation);
                    animation_player
                        .load(animation_handle.clone())
                        .play_once()
                        .unload_when_finished();
                } else {
                    log::error!("Unable to get name of item purchased");
                }
                break;
            }
        }
    }
}

pub fn generate_buy_notification_animation(name: &str) -> CharmieAnimation {
    let frame_timing = [1000.0, 130.0, 130.0, 130.0, 1000.0];
    let shade_timing = [255, 191, 127, 63, 0];
    frame_timing
        .iter()
        .zip(shade_timing)
        .map(|(&timing, shade)| {
            let mut frame_img = CharacterMapImage::new();
            let basic_color = ContentStyle::new().with(Color::Rgb {
                r: shade,
                g: shade,
                b: shade,
            });
            let emphasis_color = ContentStyle::new().with(Color::Rgb {
                r: shade / 2,
                g: shade,
                b: 0,
            });
            frame_img
                .new_row()
                .add_text("Bought ", &basic_color)
                .add_text(name, &emphasis_color);
            // .add_text("", &basic_color);
            (timing, frame_img)
        })
        .collect()
}
