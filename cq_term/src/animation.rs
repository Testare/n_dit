use std::time::Instant;

use charmi::{CharmieAnimation, CharmieAnimationFrame};
use game_core::NDitCoreSet;

use crate::prelude::*;
use crate::render::TerminalRendering;

#[derive(Copy, Clone, Debug, Default)]
pub struct AnimationPlugin;

impl Plugin for AnimationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (sys_update_animations, sys_render_animations)
                .chain()
                .before(NDitCoreSet::PostProcessCommands),
        );
    }
}

#[derive(Component, Debug)]
pub struct AnimationPlayer {
    animation: Option<Handle<CharmieAnimation>>,
    last_update: Instant,
    timing: f32,
    speed: f32,
    play_state: AnimationPlayerState,
    load_state: AnimationLoadingState,
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
            play_state: AnimationPlayerState::Paused,
            load_state: AnimationLoadingState::Unloaded,
            duration: 0.0,
            unload_when_finished: false,
        }
    }
}

impl AnimationPlayer {
    pub fn is_playing(&self) -> bool {
        matches!(
            self.play_state,
            AnimationPlayerState::Loop | AnimationPlayerState::PlayOnce
        )
    }

    fn is_loaded_or_update(
        ap: &mut Mut<AnimationPlayer>,
        assets_animations: &Assets<CharmieAnimation>,
    ) -> bool {
        match ap.load_state {
            AnimationLoadingState::Loaded => true,
            AnimationLoadingState::Unloaded => false,
            AnimationLoadingState::LoadPending => {
                if let Some(loaded_animation) = assets_animations.get(ap.animation.as_ref().expect(
                    "finalize load should not be called for animations that aren't loading",
                )) {
                    ap.duration = loaded_animation.duration();
                    ap.load_state = AnimationLoadingState::Loaded;
                    true
                } else {
                    false
                }
            },
        }
    }

    pub fn load(&mut self, handle: Handle<CharmieAnimation>) -> &mut Self {
        self.animation = Some(handle);
        self.last_update = Instant::now();
        self.load_state = AnimationLoadingState::LoadPending;
        self.play_state = AnimationPlayerState::Paused;
        self.timing = 0.0;
        self
    }

    pub fn frame(
        &self,
        assets_animation: &Assets<CharmieAnimation>,
    ) -> Option<CharmieAnimationFrame> {
        assets_animation
            .get(self.animation.as_ref()?)?
            .frame_for_timing(self.timing)
            .cloned()
    }

    pub fn unload(&mut self) {
        self.animation = None;
        self.timing = 0.0;
        self.duration = 0.0;
        self.load_state = AnimationLoadingState::Unloaded;
        self.play_state = AnimationPlayerState::Paused;
        self.unload_when_finished = false; // Reset each time
    }

    pub fn play_once(&mut self) -> &mut Self {
        self.play_state = AnimationPlayerState::PlayOnce;
        self
    }

    pub fn unload_when_finished(&mut self) -> &mut Self {
        self.unload_when_finished = true;
        self
    }

    pub fn finished(&self) -> bool {
        self.play_state == AnimationPlayerState::Finished
            || self.play_state == AnimationPlayerState::FinishedAndUnloaded
    }

    pub fn advance(&mut self) {
        // Later, can use some game time resource
        let now = Instant::now();
        let play_once = self.play_state == AnimationPlayerState::PlayOnce;
        let looping = self.play_state == AnimationPlayerState::Loop;
        let loaded = self.load_state == AnimationLoadingState::Loaded;

        if loaded && (play_once || looping) {
            let elapsed = now - self.last_update;
            self.timing += elapsed.as_millis() as f32 * self.speed / 1000.0;
            if self.timing >= self.duration {
                if play_once {
                    self.play_state = AnimationPlayerState::Finished;
                    if self.unload_when_finished {
                        self.unload();
                        self.play_state = AnimationPlayerState::FinishedAndUnloaded;
                    }
                } else if looping {
                    self.timing -= self.duration;
                }
            }
        }
        self.last_update = now;
    }
}

#[derive(Debug, Default, PartialEq)]
pub enum AnimationLoadingState {
    #[default]
    Unloaded,
    LoadPending,
    Loaded,
}

#[derive(Debug, Default, PartialEq)]
pub enum AnimationPlayerState {
    #[default]
    Paused,
    Finished,
    FinishedAndUnloaded,
    PlayOnce,
    Loop,
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
    ast_animation: Res<Assets<CharmieAnimation>>,
    mut animation_player: Query<(&mut AnimationPlayer, &mut TerminalRendering)>,
) {
    for (mut animation_player, mut tr) in animation_player.iter_mut() {
        if AnimationPlayer::is_loaded_or_update(&mut animation_player, &ast_animation) {
            tr.update_charmie(
                animation_player
                    .frame(&ast_animation)
                    .map(|frame| frame.into_charmi())
                    .unwrap_or_default(),
            );
        }
    }
}
