use std::fmt::{self, Debug};

use bevy::render::render_resource::ShaderType;

use super::CharmiImage;

#[derive(Clone, Copy, Eq, Ord, PartialEq, PartialOrd, ShaderType)]
pub struct CharCell {
    pub ch: u32,
    pub fg: u32,
    pub bg: u32,
    pub attr: u32,
}
impl CharCell {
    pub const GAP: CharCell = CharCell {
        ch: 0,
        fg: CharmiImage::NO_COLOR,
        bg: CharmiImage::NO_COLOR,
        attr: 0,
    };
    pub const BLANK: CharCell = CharCell {
        ch: 0x20,
        fg: CharmiImage::NO_COLOR,
        bg: CharmiImage::NO_COLOR,
        attr: 0,
    };

    pub fn new_ch(ch: char) -> Self {
        CharCell {
            ch: ch as u32,
            fg: 15,
            bg: 0,
            attr: 0,
        }
    }

    pub fn new_chfgbg(ch: char, fg: u32, bg: u32) -> Self {
        CharCell {
            ch: ch as u32,
            fg,
            bg,
            attr: 0,
        }
    }

    pub fn new<C: Into<u32>>(ch: C, fg: u32, bg: u32, attr: u32) -> Self {
        CharCell {
            ch: ch.into(),
            fg,
            bg,
            attr,
        }
    }

    pub fn set_fg(&mut self, fg: u32) {
        self.fg = fg;
    }

    pub fn fg(&self) -> u32 {
        self.fg
    }

    pub fn with_fg(&self, fg: u32) -> Self {
        Self { fg, ..*self }
    }

    pub fn set_bg(&mut self, bg: u32) {
        self.bg = bg;
    }

    pub fn bg(&self) -> u32 {
        self.bg
    }

    pub fn with_bg(&self, bg: u32) -> Self {
        Self { bg, ..*self }
    }

    pub fn set_attr(&mut self, attr: u32) {
        self.attr = attr;
    }

    pub fn attr(&self) -> u32 {
        self.attr
    }

    pub fn set_ch(&mut self, ch: u32) {
        self.ch = ch;
    }

    pub fn ch(&self) -> u32 {
        self.ch
    }

    pub fn draw_to(&self, dst: &Self) -> Self {
        let fg = if self.fg != CharmiImage::NO_COLOR {
            self.fg
        } else {
            dst.fg
        };
        let bg = if self.bg != CharmiImage::NO_COLOR {
            self.bg
        } else {
            dst.bg
        };
        let ch = if self.ch != 0 { self.ch } else { dst.ch };

        Self {
            ch,
            fg,
            bg,
            attr: dst.attr,
        }
    }
}

impl Debug for CharCell {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[")?;
        match self.ch {
            0 => write!(f, "◇"),
            CharmiImage::SUPPRESSED_CHAR => write!(f, "⊘"),
            ch => {
                if let Ok(ch) = <char>::try_from(ch) {
                    write!(f, "{}", ch)
                } else {
                    write!(f, "⚠[{}]", ch)
                }
            },
        }?;
        if self.fg == CharmiImage::NO_COLOR {
            write!(f, ",◇")?;
        } else if self.fg > CharmiImage::TRUE_COLOR {
            write!(f, ",#{:x}", self.fg % CharmiImage::TRUE_COLOR)?;
        } else {
            write!(f, ",{}", self.fg)?;
        };
        if self.bg == CharmiImage::NO_COLOR {
            write!(f, ",◇")?;
        } else if self.bg > CharmiImage::TRUE_COLOR {
            write!(f, ",#{:06x}", self.bg % CharmiImage::TRUE_COLOR)?;
        } else {
            write!(f, ",{}", self.bg)?;
        };
        if self.attr != 0 {
            write!(f, ",{}", self.attr)?;
        }
        write!(f, "]")
    }
}

impl Default for CharCell {
    fn default() -> Self {
        Self::GAP
    }
}

impl From<char> for CharCell {
    fn from(value: char) -> Self {
        CharCell::new_ch(value)
    }
}
