use std::time::Instant;

use charmi::{CharacterMapImage, CharmieActor, CharmieAnimation, CharmieAnimationFrame};
use crossterm::style::Stylize;
use game_core::node::{InNode, Node, NodeOp, NodePiece};
use game_core::op::OpResult;
use game_core::player::{ForPlayer, Player};
use game_core::{card, node};

use super::registry::GlyphRegistry;
use super::{NodeCursor, SelectedAction, SelectedEntity};
use crate::configuration::UiFormat;
use crate::fx::Fx;
use crate::prelude::*;
use crate::render::TerminalRendering;

// TODO Refactor this module into two separate modules: One for animation players, and
// then one under grid_ui for the attack animation specific logic
const DAMAGE_TIMING: f32 = 150.0;
const ATTACK_BASE_ANIM: &'static str = "attack";
const UNKNOWN_NODE_PIECE: &'static str = "??"; // TODO this is duplicate of render_square

#[derive(Component)]
pub struct NodeUiAttackAnimation;

#[derive(Component)]
pub struct AnimationPlayer {
    animation: Option<Handle<CharmieAnimation>>,
    last_update: Instant,
    timing: f32,
    speed: f32,
    state: AnimationPlayerState,
    duration: f32,
    unload_when_finished: bool,
}

impl Default for AnimationPlayer {
    fn default() -> Self {
        AnimationPlayer {
            animation: None,
            last_update: Instant::now(),
            timing: 0.0,
            speed: 1000.0,
            state: AnimationPlayerState::Unloaded,
            duration: 0.0,
            unload_when_finished: false,
        }
    }
}

impl AnimationPlayer {
    pub fn is_playing(&self) -> bool {
        matches!(
            self.state,
            AnimationPlayerState::Loop | AnimationPlayerState::PlayOnce
        )
    }

    fn load(
        &mut self,
        handle: Handle<CharmieAnimation>,
        assets_animations: &Assets<CharmieAnimation>,
    ) -> &mut Self {
        self.duration = assets_animations
            .get(&handle)
            .map(|animation| animation.duration())
            .unwrap_or(0.0);
        self.animation = Some(handle);
        self.last_update = Instant::now();
        self.state = AnimationPlayerState::Paused;
        self.timing = 0.0;
        self
    }

    fn frame(&self, assets_animation: &Assets<CharmieAnimation>) -> Option<CharmieAnimationFrame> {
        assets_animation
            .get(self.animation.as_ref()?)?
            .frame_for_timing(self.timing)
            .cloned()
    }

    fn unload(&mut self) {
        self.animation = None;
        self.timing = 0.0;
        self.state = AnimationPlayerState::Unloaded;
        self.unload_when_finished = false; // Reset each time
    }

    fn play_once(&mut self) -> &mut Self {
        self.state = AnimationPlayerState::PlayOnce;
        self
    }

    fn unload_when_finished(&mut self) -> &mut Self {
        self.unload_when_finished = true;
        self
    }

    fn finished(&self) -> bool {
        self.state == AnimationPlayerState::Finished
            || self.state == AnimationPlayerState::FinishedAndUnloaded
    }

    fn advance(&mut self) {
        // Later, can use some game time resource
        let now = Instant::now();
        let play_once = self.state == AnimationPlayerState::PlayOnce;
        let looping = self.state == AnimationPlayerState::Loop;
        if play_once || looping {
            let elapsed = now - self.last_update;
            self.timing += elapsed.as_millis() as f32 * self.speed / 1000.0;
            if self.timing >= self.duration {
                if play_once {
                    self.state = AnimationPlayerState::Finished;
                    if self.unload_when_finished {
                        self.unload();
                        self.state = AnimationPlayerState::FinishedAndUnloaded;
                    }
                } else if looping {
                    self.timing -= self.duration;
                }
            }
        }
        self.last_update = now;
    }
}

#[derive(Default, PartialEq)]
pub enum AnimationPlayerState {
    #[default]
    Unloaded,
    Paused,
    Finished,
    FinishedAndUnloaded,
    PlayOnce,
    Loop,
}

// This is an incredibly rough draft of how this logic should work.
pub fn sys_create_attack_animation(
    fx: Res<Fx>,
    mut ev_node_op: EventReader<OpResult<NodeOp>>,
    players: Query<(Entity, &InNode), With<Player>>,
    mut assets_animation: ResMut<Assets<CharmieAnimation>>,
    mut attack_animation_player: Query<
        (&mut AnimationPlayer, &ForPlayer),
        With<NodeUiAttackAnimation>,
    >,
    assets_actor: Res<Assets<CharmieActor>>,
    glyph_registry: Res<GlyphRegistry>,
    node_pieces: Query<&NodePiece>,
) {
    for node_op_result in ev_node_op.iter() {
        if let Op {
            op: NodeOp::PerformCurioAction { target, .. },
            ..
        } = node_op_result.source()
        {
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
                let players_in_node: HashSet<Entity> = players
                    .iter()
                    .filter(|(_, InNode(id))| *id == node_id)
                    .map(|(x, _)| x)
                    .collect();
                for (mut animation_player, ForPlayer(player)) in attack_animation_player.iter_mut()
                {
                    if players_in_node.contains(player) {
                        animation_player
                            .load(animation_handle.clone(), assets_animation.as_ref())
                            .play_once()
                            .unload_when_finished();
                    }
                }

                log::debug!("DAMAGES: {:?} HEAD: {:?}", damages, head);
                Some(())
            });
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
