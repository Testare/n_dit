mod actor;
mod char_cell;
mod image;
mod rich_text;

pub use actor::{CharmiActor, CharmiAnimation, CharmiAnimationFrame};
pub use char_cell::CharCell;
pub use image::CharmiImage;
pub use rich_text::SanitizedText;

/// For expressing the behavior for filling empty parts of an image
pub trait CharmiFill {
    fn as_fill(&self, base: &CharCell) -> CharCell;
    fn as_default_fill(&self) -> CharCell {
        self.as_fill(&CharCell::default())
    }
}

impl CharmiFill for CharCell {
    fn as_fill(&self, _base: &CharCell) -> CharCell {
        *self
    }
}

impl CharmiFill for Option<char> {
    fn as_fill(&self, base: &CharCell) -> CharCell {
        match self {
            None => CharCell::GAP,
            Some(ch) => ch.as_fill(base),
        }
    }
}

impl CharmiFill for char {
    fn as_fill(&self, base: &CharCell) -> CharCell {
        CharCell {
            ch: (*self) as u32,
            ..*base
        }
    }
}
