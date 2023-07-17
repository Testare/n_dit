use std::time::{Duration, Instant};

use crossterm::style::Stylize;
use game_core::node::{InNode, NodeOp};
use game_core::op::OpResult;
use game_core::player::{ForPlayer, Player};
use game_core::{card, node};

use crate::charmie::{
    BrokenCharacterFillBehavior, CharacterMapImage, CharmieActor, CharmieAnimation,
    CharmieAnimationFrame,
};
use crate::term::fx::Fx;
use crate::term::prelude::*;
use crate::term::render::TerminalRendering;
/*
 * I have to decide how to actually do this
 *
 * An Animation entity could have Frames as individual children?
 *
 * Then the Animation contains a
 */

const DAMAGE_TIMING: f32 = 150.0;
const ATTACK_BASE_ANIM: &'static str = "attack";

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
    ) {
        self.duration = assets_animations
            .get(&handle)
            .map(|animation| animation.duration())
            .unwrap_or(0.0);
        self.animation = Some(handle);
        self.last_update = Instant::now();
        self.state = AnimationPlayerState::Paused;
        self.timing = 0.0;
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
    }

    fn play_once(&mut self) {
        self.state = AnimationPlayerState::PlayOnce;
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
) {
    for node_op_result in ev_node_op.iter() {
        if let Op {
            op: NodeOp::PerformCurioAction { target, .. },
            ..
        } = node_op_result.source()
        {
            node_op_result.result().as_ref().ok().and_then(|metadata| {
                let damages = metadata.get_or_default(card::key::DAMAGES).ok()?;
                let fatal = metadata.get(card::key::FATAL).ok()?;
                let node_id = metadata.get(node::key::NODE_ID).ok()?;
                let head = fatal.then(|| damages.last().copied()).flatten();
                let base_animation = assets_actor
                    .get(&fx.0)
                    .map(|actor| {
                        actor
                            .animation("attack")
                            .expect("Should have attack animation")
                    })
                    .expect("FX should be loaded");
                let animation_handle = assets_animation.add(generate_animation_from_damages(
                    &damages,
                    base_animation,
                    *target,
                ));
                let players_in_node: HashSet<Entity> = players
                    .iter()
                    .filter(|(_, InNode(id))| *id == node_id)
                    .map(|(x, _)| x)
                    .collect();
                for (mut animation_player, ForPlayer(player)) in attack_animation_player.iter_mut()
                {
                    if players_in_node.contains(player) {
                        animation_player.load(animation_handle.clone(), assets_animation.as_ref());
                        animation_player.play_once();
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
        animation_player.advance();
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

fn generate_animation_from_damages(
    damages: &Vec<UVec2>,
    base_animation: &CharmieAnimation,
    target: UVec2,
) -> CharmieAnimation {
    let target = UVec2 {
        x: target.x * 3 + 1,
        y: target.y * 2 + 1,
    };
    let base_offset = UVec2 { x: 12, y: 8 };
    let damage_cell = CharacterMapImage::new()
        .with_row(|row| row.with_styled_text("[]".stylize().white().on_dark_red()));
    let damages: CharmieAnimation = (0..damages.len())
        .map(|i| {
            let mut frame = CharacterMapImage::default();
            for UVec2 { x, y } in damages.iter().skip(i) {
                frame = frame.draw(
                    &damage_cell,
                    x * 3 + 1,
                    y * 2 + 1,
                    BrokenCharacterFillBehavior::Gap,
                );
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
