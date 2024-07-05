use crate::ColorValue;

/// Represents a single space in a CharacterMapImage.
///
/// Note: If a cell containers a character that is larger than 1 cell in width,
/// it is expected that the cells that are obfuscated will be ignored.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CharmiCell {
    pub character: Option<char>,
    pub fg: Option<ColorValue>,
    pub bg: Option<ColorValue>,
    // TODO attributes
}

impl CharmiCell {
    pub const fn new_empty() -> Self {
        CharmiCell {
            character: None,
            fg: None,
            bg: None,
        }
    }

    pub const fn new_blank() -> Self {
        CharmiCell {
            character: Some(' '),
            fg: None,
            bg: None,
        }
    }
}
