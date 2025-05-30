use std::collections::HashMap;
use std::sync::LazyLock;

use crate::CharmiImage;

/// String identifiers for colors (or "no color")
/// There might be multiple names for each color, but there shouldn't be multiple
/// colors for each name.
///
/// Have not yet adopted an actual official standard yet. Thought of adopting
/// [this one](https://www.ditig.com/256-colors-cheat-sheet), but it has duplicate
/// color names (purple, purple4, etc.) so that's a bit of a non-starter.
///
/// Later identifiers will be used by default when saving images
pub static COLOR_NAME_PAIRS: &[(&str, u32)] = &[
    ("black", 0),
    ("darkred", 1),
    ("dark red", 1),
    ("darkgreen", 2),
    ("dark green", 2),
    ("darkyellow", 3),
    ("dark yellow", 3),
    ("darkblue", 4),
    ("dark blue", 4),
    ("navy", 4),
    ("darkmagenta", 5),
    ("dark magenta", 5),
    ("purple", 5),
    ("darkcyan", 6),
    ("dark cyan", 6),
    ("teal", 6),
    ("gray", 7),
    ("grey", 7),
    ("darkgray", 8),
    ("darkgrey", 8),
    ("dark gray", 8),
    ("dark grey", 8),
    ("red", 9),
    ("green", 10),
    ("lime", 10),
    ("yellow", 11),
    ("blue", 12),
    ("magenta", 13),
    ("cyan", 14),
    ("aqua", 14),
    ("aquamarine", 14),
    ("white", 15),
    ("nc", CharmiImage::NO_COLOR),
    ("nocolor", CharmiImage::NO_COLOR),
    ("no color", CharmiImage::NO_COLOR),
];

pub static COLOR_NAME_TO_CODE: LazyLock<HashMap<String, u32>> = LazyLock::new(|| {
    COLOR_NAME_PAIRS
        .iter()
        .map(|(k, v)| (k.to_string(), *v))
        .collect()
});

pub static COLOR_CODE_TO_NAME: LazyLock<HashMap<u32, String>> = LazyLock::new(|| {
    COLOR_NAME_PAIRS
        .iter()
        .map(|(k, v)| (*v, k.to_string()))
        .collect()
});

pub trait CharmiColor {
    fn as_color_u32(&self) -> u32;
}

impl CharmiColor for u32 {
    fn as_color_u32(&self) -> u32 {
        *self
    }
}

impl CharmiColor for (u8, u8, u8) {
    fn as_color_u32(&self) -> u32 {
        let (r, g, b) = *self;
        CharmiImage::TRUE_COLOR | (r as u32) << 16 | (g as u32) << 8 | b as u32
    }
}

impl CharmiColor for &str {
    fn as_color_u32(&self) -> u32 {
        COLOR_NAME_TO_CODE
            .get(&self.to_lowercase())
            .copied()
            .unwrap_or(CharmiImage::NO_COLOR)
    }
}

impl CharmiColor for Option<u32> {
    fn as_color_u32(&self) -> u32 {
        CharmiImage::NO_COLOR
    }
}

impl CharmiColor for crossterm::style::Color {
    fn as_color_u32(&self) -> u32 {
        use crossterm::style::Color;
        match self {
            Color::AnsiValue(ansi) => *ansi as u32,
            Color::Rgb { r, g, b } => {
                (*r as u32) << 16 | (*g as u32) << 8 | (*b as u32) | CharmiImage::TRUE_COLOR
            },
            Color::Reset => CharmiImage::NO_COLOR,
            Color::Black => 0,
            Color::DarkRed => 1,
            Color::DarkGreen => 2,
            Color::DarkYellow => 3,
            Color::DarkBlue => 4,
            Color::DarkMagenta => 5,
            Color::DarkCyan => 6,
            Color::Grey => 7,
            Color::DarkGrey => 8,
            Color::Red => 9,
            Color::Green => 10,
            Color::Yellow => 11,
            Color::Blue => 12,
            Color::Magenta => 13,
            Color::Cyan => 14,
            Color::White => 15,
        }
    }
}
