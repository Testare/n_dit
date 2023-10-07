use std::borrow::Borrow;
use std::ops::AddAssign;

use bevy::reflect::{TypePath, TypeUuid};
use bevy::utils::HashMap;

use super::CharacterMapImage;

#[derive(Clone, Debug, Default, PartialEq, TypeUuid, TypePath)]
#[uuid = "3dd4417c-1c8f-4ed6-9702-100b1423620a"]
#[type_path = "charmi"]
pub struct CharmieActor {
    pub(super) animations: HashMap<String, CharmieAnimation>,
}

#[derive(Clone, Debug, Default, PartialEq, TypePath, TypeUuid)]
#[uuid = "e9cccab6-b268-455d-b71b-c37b6247455b"]
#[type_path = "charmi"]
pub struct CharmieAnimation {
    pub(super) frames: Vec<CharmieAnimationFrame>,
    pub(super) timings: Vec<f32>, // f32 = last frame of animation
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct CharmieAnimationFrame {
    // Might eventually contain other metadata, such as anchor points
    pub(super) charmi: CharacterMapImage,
}

impl CharmieActor {
    pub fn insert_animation(&mut self, name: String, animation: CharmieAnimation) -> &mut Self {
        self.animations.insert(name, animation);
        self
    }

    pub fn animation<S: Borrow<str>>(&self, name: S) -> Option<&CharmieAnimation> {
        self.animations.get(name.borrow())
    }
}

impl<S: ToString> FromIterator<(S, CharmieAnimation)> for CharmieActor {
    fn from_iter<T: IntoIterator<Item = (S, CharmieAnimation)>>(iter: T) -> Self {
        Self {
            animations: HashMap::from_iter(
                iter.into_iter()
                    .map(|(name, animation)| (name.to_string(), animation)),
            ),
        }
    }
}

impl From<HashMap<String, CharmieAnimation>> for CharmieActor {
    fn from(animations: HashMap<String, CharmieAnimation>) -> Self {
        Self { animations }
    }
}

impl CharmieAnimation {
    pub fn duration(&self) -> f32 {
        self.timings.last().copied().unwrap_or_default()
    }

    pub fn image_for_timing(&self, timing: f32) -> Option<&CharacterMapImage> {
        self.frame_for_timing(timing).map(|frame| &frame.charmi)
    }

    pub fn frame_for_timing(&self, timing: f32) -> Option<&CharmieAnimationFrame> {
        self.timings
            .iter()
            .enumerate()
            .skip_while(|(_, t)| **t < timing)
            .map(|(i, _)| &self.frames[i])
            .next()
    }

    pub fn frame(&self, index: usize) -> Option<&CharmieAnimationFrame> {
        self.frames.get(index)
    }

    pub fn add_frame(&mut self, timing: f32, frame: CharmieAnimationFrame) -> &mut Self {
        let last_time = self.timings.last().cloned().unwrap_or_default();
        self.frames.push(frame);
        self.timings.push(last_time + timing);
        self
    }

    pub fn iter(&self) -> impl Iterator<Item = (f32, &CharmieAnimationFrame)> {
        self.timings
            .iter()
            .scan(0.0f32, |last_time, &current_time| {
                let timing = current_time - *last_time;
                *last_time = current_time;
                Some(timing)
            })
            .zip(self.frames.iter())
    }
}

impl IntoIterator for CharmieAnimation {
    type IntoIter = Box<dyn Iterator<Item = (f32, CharmieAnimationFrame)>>;
    type Item = (f32, CharmieAnimationFrame);
    fn into_iter(self) -> Self::IntoIter {
        Box::new(
            self.timings
                .into_iter()
                .scan(0.0f32, |last_time, current_time| {
                    let timing = current_time - *last_time;
                    *last_time = current_time;
                    Some(timing)
                })
                .zip(self.frames),
        )
    }
}

impl AddAssign<CharmieAnimation> for CharmieAnimation {
    fn add_assign(&mut self, rhs: CharmieAnimation) {
        for (timing, frame) in rhs.into_iter() {
            self.add_frame(timing, frame);
        }
    }
}

impl From<CharacterMapImage> for CharmieAnimationFrame {
    fn from(value: CharacterMapImage) -> Self {
        Self { charmi: value }
    }
}

impl FromIterator<(f32, CharacterMapImage)> for CharmieAnimation {
    fn from_iter<T: IntoIterator<Item = (f32, CharacterMapImage)>>(iter: T) -> Self {
        let mut frames = Vec::new();
        let mut timings = Vec::new();
        let mut accumulated_time: f32 = 0.0;
        for (timing, frame) in iter {
            frames.push(frame.into());
            timings.push(accumulated_time + timing);
            accumulated_time += timing;
        }
        Self { frames, timings }
    }
}

impl FromIterator<(f32, CharmieAnimationFrame)> for CharmieAnimation {
    fn from_iter<T: IntoIterator<Item = (f32, CharmieAnimationFrame)>>(iter: T) -> Self {
        let mut frames = Vec::new();
        let mut timings = Vec::new();
        let mut accumulated_time: f32 = 0.0;
        for (timing, frame) in iter {
            frames.push(frame);
            timings.push(accumulated_time + timing);
            accumulated_time += timing;
        }
        Self { frames, timings }
    }
}

impl CharmieAnimationFrame {
    pub fn charmi(&self) -> &CharacterMapImage {
        &self.charmi
    }

    pub fn into_charmi(self) -> CharacterMapImage {
        self.charmi
    }
}
