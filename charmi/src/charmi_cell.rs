use crate::ColorDef;

/// Represents a single space in a CharacterMapImage.
///
/// Note: If a cell containers a character that is larger than 1 cell in width,
/// it is expected that the cells that are obfuscated will be ignored.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct CharmiCell {
    pub character: Option<char>,
    pub fg: Option<ColorDef>,
    pub bg: Option<ColorDef>,
    // TODO attributes
}
