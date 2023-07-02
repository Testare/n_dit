use std::borrow::Borrow;
use std::fmt::Display;
use std::ops::AddAssign;

use crossterm::style::{ContentStyle, StyledContent};

#[derive(Debug, Default)]
pub enum ColorSupportLevel {
    #[default]
    TrueColor,
    Ansi256,
    Basic,
    Plain,
}

// Defines a color with optional support at different levels
pub struct CharmieColor {
    true_color: Option<(u8, u8, u8)>,
    ansi_256: Option<u8>,
    basic: Option<String>,
}

#[derive(Debug, Default)]
struct CharacterMapImage {
    rows: Vec<CharmieRow>,
}

impl CharacterMapImage {}

impl From<CharacterMapImage> for Vec<String> {
    fn from(value: CharacterMapImage) -> Self {
        value.rows.iter().map(ToString::to_string).collect()
    }
}

impl FromIterator<CharmieRow> for CharacterMapImage {
    fn from_iter<T: IntoIterator<Item = CharmieRow>>(iter: T) -> Self {
        CharacterMapImage {
            rows: iter.into_iter().collect(),
        }
    }
}

/// Represents a single row of a charmie image
#[derive(Debug, Default, Clone, PartialEq)]
struct CharmieRow {
    segments: Vec<CharmieSegment>,
}

impl CharmieRow {
    pub fn add_gap(&mut self, len: u32) {
        if let Some(CharmieSegment::Empty { len: last_len }) = self.segments.last_mut() {
            *last_len += len;
        } else {
            self.segments.push(CharmieSegment::Empty { len });
        }
    }

    pub fn add_text<S: Borrow<str>>(&mut self, text: S, text_format: &ContentStyle) {
        match self.segments.last_mut() {
            Some(CharmieSegment::Textual {
                text: last_text,
                style: format,
            }) if *format == *text_format => last_text.push_str(text.borrow()),
            _ => {
                self.segments.push(CharmieSegment::Textual {
                    text: text.borrow().into(),
                    style: *text_format,
                });
            },
        }
    }

    pub fn add_styled_text<D: Display>(&mut self, styled_content: StyledContent<D>) {
        self.add_text(styled_content.content().to_string(), styled_content.style())
    }
}

impl Display for CharmieRow {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for segment in self.segments.iter() {
            write!(f, "{}", segment)?;
        }
        Ok(())
    }
}

impl<D: Display> From<StyledContent<D>> for CharmieRow {
    fn from(value: StyledContent<D>) -> Self {
        CharmieRow {
            segments: vec![value.into()],
        }
    }
}

/// Should NOT contain newline characters
impl From<String> for CharmieRow {
    fn from(value: String) -> Self {
        CharmieRow {
            segments: vec![value.into()],
        }
    }
}

impl AddAssign<&str> for CharmieRow {
    fn add_assign(&mut self, rhs: &str) {
        self.add_text(rhs, &Default::default());
    }
}

impl<D: Display> AddAssign<StyledContent<D>> for CharmieRow {
    fn add_assign(&mut self, rhs: StyledContent<D>) {
        self.add_styled_text(rhs);
    }
}

impl AddAssign<&CharmieRow> for CharmieRow {
    fn add_assign(&mut self, rhs: &CharmieRow) {
        for segment in rhs.segments.iter() {
            match segment {
                CharmieSegment::Empty { len } => self.add_gap(*len),
                CharmieSegment::Textual {
                    text,
                    style: format,
                } => self.add_text(text.as_str(), format),
            }
        }
    }
}

/// Charmie Segment - Internal representation of a row segment
/// Cannot use StyledContent because it does not provide mutable access
/// to content
///
#[derive(Debug, Clone, PartialEq)]
enum CharmieSegment {
    Textual { text: String, style: ContentStyle },
    Empty { len: u32 },
}

impl Display for CharmieSegment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CharmieSegment::Empty { len } => {
                let lensize = *len as usize;
                write!(f, "{:lensize$}", "")
            },
            CharmieSegment::Textual { text, style } => {
                write!(f, "{}", style.apply(text))
            },
        }
    }
}

impl From<String> for CharmieSegment {
    fn from(value: String) -> Self {
        CharmieSegment::Textual {
            text: value,
            style: Default::default(),
        }
    }
}

impl<D: Display> From<StyledContent<D>> for CharmieSegment {
    fn from(value: StyledContent<D>) -> Self {
        CharmieSegment::Textual {
            text: value.content().to_string(),
            style: *value.style(),
        }
    }
}
