use std::time::Instant;

use charmi::{CharmieAnimation, CharmieAnimationFrame};

use crate::prelude::*;

#[derive(Component, Debug)]
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

    pub fn load(
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
        self.state = AnimationPlayerState::Unloaded;
        self.unload_when_finished = false; // Reset each time
    }

    pub fn play_once(&mut self) -> &mut Self {
        self.state = AnimationPlayerState::PlayOnce;
        self
    }

    pub fn unload_when_finished(&mut self) -> &mut Self {
        self.unload_when_finished = true;
        self
    }

    pub fn finished(&self) -> bool {
        self.state == AnimationPlayerState::Finished
            || self.state == AnimationPlayerState::FinishedAndUnloaded
    }

    pub fn advance(&mut self) {
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

#[derive(Debug, Default, PartialEq)]
pub enum AnimationPlayerState {
    #[default]
    Unloaded,
    Paused,
    Finished,
    FinishedAndUnloaded,
    PlayOnce,
    Loop,
}
