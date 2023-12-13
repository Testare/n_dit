use bevy::audio::VolumeLevel;
use bevy::prelude::{AudioBundle, PlaybackSettings};
use charmi::{CharacterMapImage, CharmieActor, CharmieAnimation};
use crossterm::style::Stylize;
use game_core::node::{InNode, NodeOp, NodePiece};
use game_core::op::OpResult;
use game_core::player::{ForPlayer, Player};
use game_core::registry::Reg;
use game_core::{card, node};

use crate::animation::AnimationPlayer;
use crate::fx::Fx;
use crate::node_ui::NodeGlyph;
use crate::prelude::*;

const DAMAGE_TIMING: f32 = 150.0;
const ATTACK_BASE_ANIM: &str = "attack";
const PICKUP_BASE_ANIM: &str = "pickup";

#[derive(Component)]
pub struct GridUiAnimation;

pub fn sys_grid_animations(
    mut commands: Commands,
    fx: Res<Fx>,
    mut ev_node_op: EventReader<OpResult<NodeOp>>,
    players: Query<(Entity, &InNode), With<Player>>,
    mut assets_animation: ResMut<Assets<CharmieAnimation>>,
    mut grid_animation_player: Query<(&mut AnimationPlayer, &ForPlayer), With<GridUiAnimation>>,
    assets_actor: Res<Assets<CharmieActor>>,
    reg_glyph: Res<Reg<NodeGlyph>>,
    node_pieces: Query<&NodePiece>,
) {
    for node_op_result in ev_node_op.read() {
        if let Some((node_id, animation_handle)) = match node_op_result.op() {
            NodeOp::PerformCurioAction { target, .. } => {
                node_op_result.result().as_ref().ok().and_then(|metadata| {
                    let effects_metadata = metadata.get_or_default(node::key::EFFECTS).ok()?;
                    let damages = effects_metadata.get_or_default(card::key::DAMAGES).ok()?;
                    if damages.is_empty() {
                        return None;
                    }
                    let fatal = effects_metadata.get_or_default(card::key::FATAL).ok()?;
                    let node_id = metadata.get_required(node::key::NODE_ID).ok()?;
                    let target_head = if fatal {
                        effects_metadata
                            .get_optional(card::key::TARGET_ENTITY)
                            .ok()
                            .flatten()
                            .and_then(|target_id| {
                                let display_id = get_assert!(target_id, node_pieces)?.display_id();
                                let head_str = reg_glyph
                                    .get(display_id)
                                    .cloned()
                                    .unwrap_or_default()
                                    .glyph();
                                Some(head_str)
                            })
                    } else {
                        None
                    };
                    let head = fatal.then(|| damages.last().copied()).flatten();
                    let base_animation = assets_actor
                        .get(&fx.charmia)
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

                    let glyph = reg_glyph
                        .get(pickup.default_diplay_id())
                        .cloned()
                        .unwrap_or_default();
                    let pickup_display = CharacterMapImage::new()
                        .with_row(|row| row.with_styled_text(glyph.styled_glyph()));

                    let base_animation = assets_actor
                        .get(&fx.charmia)
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
                    commands.spawn(AudioBundle {
                        source: fx.pickup_sound.clone(),
                        settings: PlaybackSettings {
                            mode: bevy::audio::PlaybackMode::Despawn,
                            volume: bevy::audio::Volume::Relative(VolumeLevel::new(13.0)),
                            ..default()
                        },
                    });

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
                        .load(animation_handle.clone())
                        .play_once()
                        .unload_when_finished();
                }
            }
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
                    frame = frame.draw(target_head, x * 3 + 1, y * 2 + 1, None);
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
