use bevy::reflect::TypeUuid;
use bevy::utils::HashMap;

use super::CharacterMapImage;

#[derive(Clone, Debug, TypeUuid)]
#[uuid = "3dd4417c-1c8f-4ed6-9702-100b1423620a"]
struct CharmieActor {
    animations: HashMap<String, CharmieAnimation>,
}

#[derive(Clone, Debug)]
struct CharmieAnimation {
    frames: Vec<CharmieAnimationFrame>,
    timings: Vec<(f32, usize)>, // f32 = last frame of animation
}

#[derive(Clone, Debug)]
struct CharmieAnimationFrame {
    charmi: CharacterMapImage,
}

impl CharmieAnimation {
    pub fn image_for_timing(&self, timing: f32) -> Option<&CharacterMapImage> {
        self.frame_for_timing(timing).map(|frame| &frame.charmi)
    }
    fn frame_for_timing(&self, timing: f32) -> Option<&CharmieAnimationFrame> {
        self.timings
            .iter()
            .skip_while(|(t, _)| *t < timing)
            .map(|(_, i)| &self.frames[*i])
            .next()
    }
}
