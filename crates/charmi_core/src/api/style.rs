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

impl std::ops::Add<crossterm::style::ContentStyle> for CharmiStyle {
    type Output = CharmiStyle;
    fn add(self, rhs: crossterm::style::ContentStyle) -> Self::Output {
        self + &rhs
    }
}
impl std::ops::Add<&crossterm::style::ContentStyle> for CharmiStyle {
    type Output = CharmiStyle;
    fn add(self, rhs: &crossterm::style::ContentStyle) -> Self::Output {
        let reverse = rhs.attributes.has(crossterm::style::Attribute::Reverse);
        let add = self + CharmiStyle::from(rhs);
        if reverse {
            // Until we actually add attributes
            CharmiStyle {
                bg: add.fg,
                fg: add.bg,
                attr: add.attr,
            }
        } else {
            add
        }
    }
}

impl std::ops::Add<CharmiStyle> for CharmiStyle {
    type Output = CharmiStyle;
    fn add(self, rhs: CharmiStyle) -> Self::Output {
        (&self) + rhs
    }
}

impl std::ops::Add<CharmiStyle> for &CharmiStyle {
    type Output = CharmiStyle;
    fn add(self, rhs: CharmiStyle) -> Self::Output {
        CharmiStyle {
            fg: (rhs.fg == CharmiImage::NO_COLOR)
                .then_some(self.fg)
                .unwrap_or(rhs.fg),
            bg: (rhs.bg == CharmiImage::NO_COLOR)
                .then_some(self.bg)
                .unwrap_or(rhs.bg),
            attr: (rhs.attr == CharmiImage::NO_COLOR)
                .then_some(self.attr)
                .unwrap_or(rhs.attr),
        }
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

impl From<crossterm::style::ContentStyle> for CharmiStyle {
    fn from(value: crossterm::style::ContentStyle) -> Self {
        CharmiStyle::from(&value)
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
