use charmi::{CharacterMapImage, CharmieActor, CharmieAnimation};
use crossterm::style::Stylize;
use game_core::node::{InNode, Node, NodeOp, NodePiece};
use game_core::op::OpResult;
use game_core::player::{ForPlayer, Player};
use game_core::{card, node};

use super::super::registry::GlyphRegistry;
use super::{NodeCursor, SelectedAction, SelectedEntity};
use crate::animation::AnimationPlayer;
use crate::configuration::UiFormat;
use crate::fx::Fx;
use crate::node_ui::ShowNode;
use crate::prelude::*;
use crate::render::TerminalRendering;

// TODO Refactor this module into two separate modules: One for animation players, and
// then one under grid_ui for the grid animation specific logic
const DAMAGE_TIMING: f32 = 150.0;
const ATTACK_BASE_ANIM: &'static str = "attack";
const PICKUP_BASE_ANIM: &'static str = "pickup";
const UNKNOWN_NODE_PIECE: &'static str = "??"; // TODO this is duplicate of render_square

#[derive(Component)]
pub struct GridUiAnimation;

// This is an incredibly rough draft of how this logic should work.
pub fn sys_grid_animations(
    fx: Res<Fx>,
    mut ev_node_op: EventReader<OpResult<NodeOp>>,
    players: Query<(Entity, &InNode), With<Player>>,
    mut assets_animation: ResMut<Assets<CharmieAnimation>>,
    mut grid_animation_player: Query<(&mut AnimationPlayer, &ForPlayer), With<GridUiAnimation>>,
    assets_actor: Res<Assets<CharmieActor>>,
    glyph_registry: Res<GlyphRegistry>,
    node_pieces: Query<&NodePiece>,
) {
    for node_op_result in ev_node_op.iter() {
        if let Some((node_id, animation_handle)) = match node_op_result.source().op() {
            NodeOp::PerformCurioAction { target, .. } => {
                node_op_result.result().as_ref().ok().and_then(|metadata| {
                    let damages = metadata.get_or_default(card::key::DAMAGES).ok()?;
                    if damages.is_empty() {
                        return None;
                    }
                    let fatal = metadata.get_or_default(card::key::FATAL).ok()?;
                    let node_id = metadata.get_required(node::key::NODE_ID).ok()?;
                    let target_head = if fatal {
                        metadata
                            .get_optional(card::key::TARGET_ENTITY)
                            .ok()
                            .flatten()
                            .and_then(|target_id| {
                                let display_id = get_assert!(target_id, node_pieces)?.display_id();
                                let (head_str, _) = glyph_registry
                                    .get(display_id)
                                    .cloned()
                                    .unwrap_or((UNKNOWN_NODE_PIECE.to_owned(), UiFormat::NONE));
                                Some(head_str)
                                // Some(())
                            })
                    } else {
                        None
                    };
                    let head = fatal.then(|| damages.last().copied()).flatten();
                    let base_animation = assets_actor
                        .get(&fx.0)
                        .map(|actor| {
                            actor
                                .animation(ATTACK_BASE_ANIM)
                                .expect("Should have attack animation")
                        })
                        .expect("FX should be loaded");
                    let animation_handle = assets_animation.add(generate_animation_from_damages(
                        &damages,
                        base_animation,
                        *target,
                        target_head,
                    ));
                    log::debug!("DAMAGES: {:?} HEAD: {:?}", damages, head);
                    Some((node_id, animation_handle))
                })
            },
            NodeOp::MoveActiveCurio { .. } => {
                node_op_result.result().as_ref().ok().and_then(|metadata| {
                    let pickup = metadata.get_optional(node::key::PICKUP).ok().flatten()?;
                    let node_id = metadata.get_required(node::key::NODE_ID).ok()?;
                    let target_pt = metadata.get_required(node::key::TARGET_POINT).ok()?;

                    let (pickup_display, format) = glyph_registry
                        .get(pickup.default_diplay_id())
                        .cloned()
                        .unwrap_or((UNKNOWN_NODE_PIECE.to_owned(), UiFormat::NONE));
                    let pickup_display = CharacterMapImage::new()
                        .with_row(|row| row.with_styled_text(format.apply(pickup_display)));

                    let base_animation = assets_actor
                        .get(&fx.0)
                        .map(|actor| {
                            actor
                                .animation(PICKUP_BASE_ANIM)
                                .expect("Should have pickup animation")
                        })
                        .expect("FX should be loaded");
                    let animation_handle = assets_animation.add(generate_pickup_animation(
                        base_animation,
                        target_pt,
                        pickup_display,
                    ));

                    Some((node_id, animation_handle))
                })
            },
            _ => None,
        } {
            let players_in_node: HashSet<Entity> = players
                .iter()
                .filter(|(_, InNode(id))| *id == node_id)
                .map(|(x, _)| x)
                .collect();
            for (mut animation_player, ForPlayer(player)) in grid_animation_player.iter_mut() {
                if players_in_node.contains(player) {
                    animation_player
                        .load(animation_handle.clone(), assets_animation.as_ref())
                        .play_once()
                        .unload_when_finished();
                }
            }
        }
    }
}

pub fn sys_update_animations(mut animation_player: Query<&mut AnimationPlayer>) {
    for mut animation_player in animation_player.iter_mut() {
        if animation_player.is_playing() {
            // Do the check here so that change detection can work
            animation_player.advance();
        }
    }
}

pub fn sys_render_animations(
    animation: Res<Assets<CharmieAnimation>>,
    mut animation_player: Query<(&AnimationPlayer, &mut TerminalRendering)>,
) {
    for (animation_player, mut tr) in animation_player.iter_mut() {
        tr.update_charmie(
            animation_player
                .frame(&animation)
                .map(|frame| frame.into_charmi())
                .unwrap_or_default(),
        );
    }
}

pub fn sys_reset_state_after_animation_plays(
    changed_aps: Query<(&AnimationPlayer, &ForPlayer), Changed<AnimationPlayer>>,
    mut players: Query<
        (
            &mut SelectedAction,
            &mut SelectedEntity,
            &NodeCursor,
            &InNode,
        ),
        With<Player>,
    >,
    nodes: Query<(&EntityGrid,), With<Node>>,
) {
    for (ap, for_player) in changed_aps.iter() {
        if ap.finished() {
            get_assert_mut!(**for_player, players, |(
                mut selected_action,
                selected_entity,
                node_cursor,
                in_node,
            )| {
                let (grid,) = get_assert!(**in_node, &nodes)?;
                **selected_action = None;
                node_cursor.adjust_to_self(selected_entity, selected_action, grid);
                Some(())
            });
        }
    }
}

fn generate_animation_from_damages(
    damages: &Vec<UVec2>,
    base_animation: &CharmieAnimation,
    target: UVec2,
    target_head: Option<String>,
) -> CharmieAnimation {
    let target = UVec2 {
        x: target.x * 3 + 1,
        y: target.y * 2 + 1,
    };
    let base_offset = UVec2 { x: 12, y: 8 };
    let damage_cell = CharacterMapImage::new()
        .with_row(|row| row.with_styled_text("[]".stylize().white().on_dark_red()));
    let target_head = target_head.map(|target_head_str| {
        CharacterMapImage::new()
            .with_row(|row| row.with_styled_text(target_head_str.stylize().white().on_dark_red()))
    });
    let damages: CharmieAnimation = (0..damages.len())
        .map(|i| {
            let mut frame = CharacterMapImage::default();
            for (i, UVec2 { x, y }) in damages.iter().enumerate().skip(i) {
                if let (Some(target_head), true) = (&target_head, i == damages.len() - 1) {
                    frame = frame.draw(&target_head, x * 3 + 1, y * 2 + 1, None);
                } else {
                    frame = frame.draw(&damage_cell, x * 3 + 1, y * 2 + 1, None);
                }
            }
            (DAMAGE_TIMING, frame)
        })
        .collect();
    let full_damage_charmi = damages.frame(0).cloned().unwrap_or_default().into_charmi();
    let mut generated_animation = base_animation
        .iter()
        .map(|(timing, frame)| {
            let clipped_charmi = frame.charmi().clip(
                base_offset.x.saturating_sub(target.x),
                base_offset.y.saturating_sub(target.y),
                1024,
                1024,
                Default::default(),
            );
            let drawn_charmi = full_damage_charmi.draw(
                &clipped_charmi,
                target.x.saturating_sub(base_offset.x),
                target.y.saturating_sub(base_offset.y),
                Default::default(),
            );
            (timing, drawn_charmi)
        })
        .collect();
    generated_animation += damages;
    generated_animation
}

fn generate_pickup_animation(
    base_animation: &CharmieAnimation,
    target: UVec2,
    pickup_display: CharacterMapImage,
) -> CharmieAnimation {
    let target = UVec2 {
        x: target.x * 3 + 1,
        y: target.y * 2 + 1,
    };
    let base_offset = UVec2 { x: 12, y: 8 };
    let full_damage_charmi = CharacterMapImage::new();
    let mut generated_animation = base_animation
        .iter()
        .map(|(timing, frame)| {
            let clipped_charmi = frame.charmi().clip(
                base_offset.x.saturating_sub(target.x),
                base_offset.y.saturating_sub(target.y),
                1024,
                1024,
                Default::default(),
            );
            let drawn_charmi = full_damage_charmi.draw(
                &clipped_charmi,
                target.x.saturating_sub(base_offset.x),
                target.y.saturating_sub(base_offset.y),
                Default::default(),
            );
            (timing, drawn_charmi)
        })
        .collect();
    let float_animation = (0..target.y)
        .rev()
        .map(|y| {
            (
                (1000.0 / target.y as f32),
                CharacterMapImage::new().draw(&pickup_display, target.x, y, None),
            )
        })
        .collect();
    generated_animation += float_animation;
    generated_animation
}

pub fn sys_create_grid_animation_player(
    mut commands: Commands,
    mut ev_show_node: EventReader<ShowNode>,
) {
    for show_node in ev_show_node.iter() {
        commands.spawn((
            Name::new("GridAnimationPlayer"),
            GridUiAnimation,
            ForPlayer(show_node.player),
            AnimationPlayer::default(),
            TerminalRendering::default(),
        ));
    }
}
