use crate::{CharCell, CharmiImage};

use super::CharmiColor;

/// Basically just a CharCell without a char.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct CharmiStyle {
    pub fg: u32,
    pub bg: u32,
    pub attr: u32,
}

impl CharmiStyle {
    /// Note, ch should be single width
    pub fn of_ch(&self, ch: u32) -> CharCell {
        let CharmiStyle { fg, bg, attr } = *self;
        CharCell { ch, fg, bg, attr }
    }
}

impl Default for CharmiStyle {
    fn default() -> Self {
        CharmiStyle {
            fg: CharmiImage::NO_COLOR,
            bg: CharmiImage::NO_COLOR,
            attr: 0,
        }
    }
}

impl From<&crossterm::style::ContentStyle> for CharmiStyle {
    fn from(value: &crossterm::style::ContentStyle) -> Self {
        CharmiStyle {
            fg: value
                .foreground_color
                .map(|fg| fg.as_color_u32())
                .unwrap_or(CharmiImage::NO_COLOR),
            bg: value
                .background_color
                .map(|fg| fg.as_color_u32())
                .unwrap_or(CharmiImage::NO_COLOR),
            attr: 0, // TODO
        }
    }
}
