use core::fmt;

use bevy::prelude::Deref;
use unicode_width::UnicodeWidthChar;

use crate::style::CharmiStyle;

use super::{CharCell, CharmiImage};

#[derive(Clone, PartialEq, PartialOrd, Eq, Ord, Deref)]
pub struct SanitizedText(Vec<CharCell>);

impl SanitizedText {
    pub fn from_text(text: &str, base_cell: Option<CharmiStyle>) -> Self {
        Self::from_text_full(text, base_cell, None, None)
    }

    pub fn from_text_full(
        text: &str,
        base_cell: Option<CharmiStyle>,
        split_chars: Option<(u32, u32)>,
        empty_char: Option<char>,
    ) -> Self {
        let base_cell = base_cell.unwrap_or_default();
        let split_char = split_chars.unwrap_or((' ' as u32, ' ' as u32));
        let mut sanitized_text = Vec::new();
        for ch in text.chars() {
            if Some(ch) == empty_char {
                sanitized_text.push(CharCell::GAP);
                continue;
            }
            match ch.width() {
                Some(2) => {
                    sanitized_text.push(base_cell.of_ch(ch as u32));
                    sanitized_text.push(CharCell {
                        ch: CharmiImage::SUPPRESSED_CHAR,
                        bg: split_char.0,
                        fg: split_char.1,
                        attr: base_cell.attr,
                    });
                },
                Some(1) => {
                    sanitized_text.push(base_cell.of_ch(ch as u32));
                },
                None | Some(0) => {},
                _ => panic!("Unexpected result for character width"),
            }
        }
        Self(sanitized_text)
    }

    pub fn unwrap(self) -> Vec<CharCell> {
        self.0
    }
}

impl core::fmt::Debug for SanitizedText {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let debug_text: String = self
            .0
            .iter()
            .map(|cell| match char::from_u32(cell.ch) {
                Some('\0') => '◇',
                None => '⊘',
                Some(ch) => ch,
            })
            .collect();
        write!(f, "SanitizedText({debug_text})")
    }
}
