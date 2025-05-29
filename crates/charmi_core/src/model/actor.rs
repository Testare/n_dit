use std::borrow::Borrow;
use std::collections::HashMap;
use std::ops::AddAssign;

use bevy::prelude::Asset;
use bevy::reflect::TypePath;

use super::CharmiImage;

#[derive(Asset, Clone, Debug, Default, PartialEq, TypePath)]
#[type_path = "charmi"]
pub struct CharmiActor {
    pub animations: HashMap<String, CharmiAnimation>,
}

#[derive(Asset, Clone, Debug, Default, PartialEq, TypePath)]
#[type_path = "charmi"]
pub struct CharmiAnimation {
    pub frames: Vec<CharmiAnimationFrame>,
    pub timings: Vec<f32>, // f32 = last frame of animation
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct CharmiAnimationFrame {
    // Might eventually contain other metadata, such as anchor points
    pub charmi: CharmiImage,
}

impl CharmiActor {
    pub fn insert_animation(&mut self, name: String, animation: CharmiAnimation) -> &mut Self {
        self.animations.insert(name, animation);
        self
    }

    pub fn animation<S: Borrow<str>>(&self, name: S) -> Option<&CharmiAnimation> {
        self.animations.get(name.borrow())
    }
}

impl<S: ToString> FromIterator<(S, CharmiAnimation)> for CharmiActor {
    fn from_iter<T: IntoIterator<Item = (S, CharmiAnimation)>>(iter: T) -> Self {
        Self {
            animations: HashMap::from_iter(
                iter.into_iter()
                    .map(|(name, animation)| (name.to_string(), animation)),
            ),
        }
    }
}

impl From<HashMap<String, CharmiAnimation>> for CharmiActor {
    fn from(animations: HashMap<String, CharmiAnimation>) -> Self {
        Self { animations }
    }
}

impl CharmiAnimation {
    pub fn duration(&self) -> f32 {
        self.timings.last().copied().unwrap_or_default()
    }

    pub fn image_for_timing(&self, timing: f32) -> Option<&CharmiImage> {
        self.frame_for_timing(timing).map(|frame| &frame.charmi)
    }

    pub fn frame_for_timing(&self, timing: f32) -> Option<&CharmiAnimationFrame> {
        self.timings
            .iter()
            .enumerate()
            .skip_while(|(_, t)| **t <= timing)
            .map(|(i, _)| &self.frames[i])
            .next()
    }

    pub fn frame(&self, index: usize) -> Option<&CharmiAnimationFrame> {
        self.frames.get(index)
    }

    pub fn add_frame(&mut self, timing: f32, frame: CharmiAnimationFrame) -> &mut Self {
        let last_time = self.timings.last().cloned().unwrap_or_default();
        self.frames.push(frame);
        self.timings.push(last_time + timing);
        self
    }

    pub fn iter(&self) -> impl Iterator<Item = (f32, &CharmiAnimationFrame)> {
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

impl IntoIterator for CharmiAnimation {
    type IntoIter = Box<dyn Iterator<Item = (f32, CharmiAnimationFrame)>>;
    type Item = (f32, CharmiAnimationFrame);
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

impl AddAssign<CharmiAnimation> for CharmiAnimation {
    fn add_assign(&mut self, rhs: CharmiAnimation) {
        for (timing, frame) in rhs.into_iter() {
            self.add_frame(timing, frame);
        }
    }
}

impl From<CharmiImage> for CharmiAnimationFrame {
    fn from(value: CharmiImage) -> Self {
        Self { charmi: value }
    }
}

impl FromIterator<(f32, CharmiImage)> for CharmiAnimation {
    fn from_iter<T: IntoIterator<Item = (f32, CharmiImage)>>(iter: T) -> Self {
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

impl FromIterator<(f32, CharmiAnimationFrame)> for CharmiAnimation {
    fn from_iter<T: IntoIterator<Item = (f32, CharmiAnimationFrame)>>(iter: T) -> Self {
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

impl CharmiAnimationFrame {
    pub fn charmi(&self) -> &CharmiImage {
        &self.charmi
    }

    pub fn into_charmi(self) -> CharmiImage {
        self.charmi
    }
}
