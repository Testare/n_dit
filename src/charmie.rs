use std::borrow::Borrow;
use std::fmt::Display;
use std::ops::AddAssign;

use crossterm::style::{ContentStyle, StyledContent};
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

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

#[derive(Clone, Debug, Default, PartialEq)]
pub struct CharacterMapImage {
    rows: Vec<CharmieRow>,
}

#[derive(Clone, Copy, Debug)]
pub enum BrokenCharacterFillBehavior {
    Char(char),
    Gap,
}

impl Default for BrokenCharacterFillBehavior {
    fn default() -> Self {
        Self::Char(' ')
    }
}

impl CharacterMapImage {
    pub fn new() -> Self {
        Self::default()
    }

    // Draws map onto this image, expanding the image as necessary, at the location specified.
    pub fn draw(
        &self,
        map: &CharacterMapImage,
        x: u32,
        y: u32,
        bcfb: BrokenCharacterFillBehavior,
    ) -> Self {
        let y = y as usize;
        let mut result = self.clone();

        if result.rows.len() < (y + map.rows.len()) {
            result
                .rows
                .resize(y + map.rows.len(), CharmieRow::default());
        }
        for (row_index, row) in map.rows.iter().enumerate() {
            let row_index = row_index + y as usize;
            result.rows[row_index] = result.rows[row_index].draw(row, x, bcfb);
        }
        result
    }

    pub fn fit_to_size(&mut self, width: u32, height: u32) {
        self.rows.truncate(height as usize);
        for row in self.rows.iter_mut() {
            row.pad_to(width);
        }
    }

    pub fn clip(
        &self,
        x: u32,
        y: u32,
        width: u32,
        height: u32,
        bcfb: BrokenCharacterFillBehavior,
    ) -> Self {
        CharacterMapImage {
            rows: self
                .rows
                .iter()
                .skip(y as usize)
                .take(height as usize)
                .map(|row| row.clip(x, width, bcfb))
                .collect(),
        }
    }

    pub fn push_row(&mut self, row: CharmieRow) -> &mut Self {
        self.rows.push(row);
        self
    }
}

impl From<&CharacterMapImage> for Vec<String> {
    fn from(value: &CharacterMapImage) -> Self {
        value.rows.iter().map(ToString::to_string).collect()
    }
}

impl From<Vec<String>> for CharacterMapImage {
    fn from(value: Vec<String>) -> Self {
        CharacterMapImage {
            rows: value.into_iter().map(|row| row.into()).collect(),
        }
    }
}

impl<T: Into<CharmieRow>> FromIterator<T> for CharacterMapImage {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        CharacterMapImage {
            rows: iter.into_iter().map(|row| row.into()).collect(),
        }
    }
}

/// Represents a single row of a charmie image
#[derive(Debug, Default, Clone, PartialEq)]
pub struct CharmieRow {
    segments: Vec<CharmieSegment>,
    // cache: OnceCell<String>, Cow?
}

impl CharmieRow {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn len(&self) -> u32 {
        self.segments
            .iter()
            .map(|segment| {
                let len = segment.len();
                len
            })
            .sum()
    }

    pub fn add_gap(&mut self, len: u32) -> &mut Self {
        if let Some(CharmieSegment::Empty { len: last_len }) = self.segments.last_mut() {
            *last_len += len;
        } else {
            self.segments.push(CharmieSegment::Empty { len });
        }
        self
    }

    pub fn add_plain_text<S: Borrow<str>>(&mut self, text: S) -> &mut Self {
        self.add_text(text, &ContentStyle::new());
        self
    }

    pub fn add_text<S: Borrow<str>>(&mut self, text: S, text_format: &ContentStyle) -> &mut Self {
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
        self
    }

    pub fn add_styled_text<D: Display>(&mut self, styled_content: StyledContent<D>) -> &mut Self {
        self.add_text(styled_content.content().to_string(), styled_content.style());
        self
    }

    pub fn with_gap(mut self, len: u32) -> Self {
        self.add_gap(len);
        self
    }

    pub fn with_plain_text(mut self, text: &str) -> Self {
        self.add_plain_text(text);
        self
    }

    pub fn with_text(mut self, text: &str, text_format: &ContentStyle) -> Self {
        self.add_text(text, text_format);
        self
    }

    pub fn with_styled_text<D: Display>(mut self, styled_content: StyledContent<D>) -> Self {
        self.add_styled_text(styled_content);
        self
    }

    pub fn fit_to_len(&mut self, len: u32) -> &mut Self {
        let self_len = self.len();
        if self_len > len {
            *self = self.clip(0, len, BrokenCharacterFillBehavior::Gap);
        } else if self_len < len {
            self.add_gap(len - self_len);
        }
        self
    }

    pub fn pad_to(&mut self, len: u32) -> &mut Self {
        let self_len = self.len();
        if self_len < len {
            self.add_gap(len - self_len);
        }
        self
    }

    pub fn draw(&self, row: &CharmieRow, x: u32, bcfb: BrokenCharacterFillBehavior) -> Self {
        let mut result = if x > 0 {
            self.clip(0, x, bcfb)
        } else {
            CharmieRow::default()
        };
        let self_len = self.len();
        if self_len < x {
            result.add_gap(x - self_len);
        }

        let mut result_len = result.len();
        for segment in row.segments.iter() {
            match segment {
                CharmieSegment::Empty { .. } => {
                    let under_clip = self.clip(result_len, segment.len(), bcfb);
                    let clip_len = under_clip.len();
                    result += &under_clip;
                    if clip_len < segment.len() {
                        result.add_gap(segment.len() - clip_len);
                    }
                },
                CharmieSegment::Textual { .. } => {
                    result += segment;
                },
            }
            result_len += segment.len();
        }

        // result += row; // Will need to fill gaps from below, that'll come later

        let result_len = result.len();
        if result_len < self_len {
            result += &self.clip(result_len, self_len - result_len, bcfb);
        }
        result
    }

    /// Note: FillBehavior only applies to character that get cut in half.
    /// It does not fill in empty space in the clip, such as if the width is bigger than the source's width
    pub fn clip(&self, clip_start: u32, width: u32, bcfb: BrokenCharacterFillBehavior) -> Self {
        let clip_end = clip_start + width;
        self.segments
            .iter()
            .fold(
                (0, CharmieRow::default()),
                |(seg_start, mut row), segment| {
                    let seg_end = seg_start + segment.len();
                    if seg_start < clip_end && clip_start < seg_end {
                        if seg_start >= clip_start && clip_end >= seg_end {
                            row += segment;
                        } else {
                            let skip_start = clip_start.saturating_sub(seg_start);
                            let take_until = clip_end.saturating_sub(seg_start).min(segment.len());
                            match segment {
                                CharmieSegment::Empty { .. } => {
                                    row += &CharmieSegment::Empty {
                                        len: take_until - skip_start,
                                    }
                                },
                                CharmieSegment::Textual { text, style } => {
                                    let text: String = text
                                        .chars()
                                        .scan(0, |index, ch| {
                                            let current_idx = *index;
                                            let char_width = ch.width().unwrap_or(0) as u32;
                                            let next_idx = current_idx + char_width;
                                            *index = next_idx;

                                            if next_idx <= skip_start {
                                                return Some(None); // Haven't gotten to the clip yet
                                            } else if current_idx >= take_until {
                                                return None; // Clip is done
                                            }

                                            if char_width == 2 {
                                                if current_idx + 1 == skip_start {
                                                    // Full width character sliced in half at the beginning
                                                    match bcfb {
                                                        BrokenCharacterFillBehavior::Char(
                                                            fill_ch,
                                                        ) => return Some(Some(fill_ch)),
                                                        BrokenCharacterFillBehavior::Gap => {
                                                            row +=
                                                                &CharmieSegment::Empty { len: 1 };
                                                            return Some(None);
                                                        },
                                                    }
                                                } else if current_idx + 1 == take_until {
                                                    // Full width character sliced in half at the end
                                                    match bcfb {
                                                        BrokenCharacterFillBehavior::Char(
                                                            fill_ch,
                                                        ) => return Some(Some(fill_ch)),
                                                        BrokenCharacterFillBehavior::Gap => {
                                                            return None;
                                                        },
                                                    }
                                                }
                                            }
                                            Some(Some(ch))
                                        })
                                        .filter_map(|c| c)
                                        .collect();

                                    row += CharmieSegment::Textual {
                                        text,
                                        style: *style,
                                    };
                                },
                            }
                        }
                    }
                    (seg_start + segment.len(), row)
                },
            )
            .1
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

impl From<&str> for CharmieRow {
    fn from(value: &str) -> Self {
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
            *self += segment;
        }
    }
}

impl AddAssign<CharmieRow> for CharmieRow {
    fn add_assign(&mut self, rhs: CharmieRow) {
        for segment in rhs.segments.iter() {
            *self += segment;
        }
    }
}

impl AddAssign<&CharmieSegment> for CharmieRow {
    fn add_assign(&mut self, rhs: &CharmieSegment) {
        match rhs {
            CharmieSegment::Empty { len } => self.add_gap(*len),
            CharmieSegment::Textual {
                text,
                style: format,
            } => self.add_text(text.as_str(), format),
        };
    }
}

impl AddAssign<CharmieSegment> for CharmieRow {
    fn add_assign(&mut self, rhs: CharmieSegment) {
        match rhs {
            CharmieSegment::Empty { len } => self.add_gap(len),
            CharmieSegment::Textual {
                text,
                style: format,
            } => self.add_text(text.as_str(), &format),
        };
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
    // Effect { len: u32, style: ContentStyle},
}

impl CharmieSegment {
    fn len(&self) -> u32 {
        match self {
            Self::Textual { text, .. } => text.width() as u32,
            Self::Empty { len } => *len,
        }
    }
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

impl From<&str> for CharmieSegment {
    fn from(value: &str) -> Self {
        CharmieSegment::Textual {
            text: value.into(),
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

#[cfg(test)]
mod tests {
    use crossterm::style::Stylize;

    use super::*;

    #[test]
    fn test_clipping_charmie_row_with_fullwidth_characters() {
        let row = CharmieRow::from("世Hello界world".to_string());

        let clipped = row.clip(0, 9, BrokenCharacterFillBehavior::Gap);
        assert_eq!(clipped.to_string(), "世Hello界");

        let clipped = row.clip(1, 7, BrokenCharacterFillBehavior::Char('x'));
        assert_eq!(clipped.to_string(), "xHellox");

        let clipped = row.clip(1, 7, BrokenCharacterFillBehavior::Gap);
        assert_eq!(clipped.to_string(), " Hello"); // Gap is not added to the end, but at the beginning
    }

    #[test]
    fn test_clipping_charmie_row_with_two_segments() {
        let row = CharmieRow::from("Hello".green()).with_plain_text("There!");
        let mut expected = "Hello".green().to_string();
        expected.push_str("There!");
        assert_eq!(row.to_string(), expected); // Gap is not added to the end, but at the beginning

        let row = CharmieRow::from("Hello".green()).with_plain_text("There!");
        let clipped = row.clip(0, 10, BrokenCharacterFillBehavior::Gap);
        let mut expected = "Hello".green().to_string();
        expected.push_str("There");
        assert_eq!(clipped.to_string(), expected); // Gap is not added to the end, but at the beginning

        let row = CharmieRow::from("Hello".green()).with_plain_text("There!");
        let clipped = row.clip(6, 4, BrokenCharacterFillBehavior::Gap);
        assert_eq!(clipped.to_string(), "here"); // Gap is not added to the end, but at the beginning
    }

    #[test]
    fn test_draw_charmie_row() {
        let row = CharmieRow::from("世Hello界world".to_string());
        let draw_row = CharmieRow::from("Mimsy".to_string());
        let gap_draw_row = CharmieRow::from("[".to_string())
            .with_gap(2)
            .with_text("]", &Default::default());

        let drawing = row.draw(&draw_row, 0, BrokenCharacterFillBehavior::Char('_'));
        assert_eq!(drawing.to_string(), "Mimsylo界world");

        let drawing = row.draw(&draw_row, 1, BrokenCharacterFillBehavior::Char('_'));
        assert_eq!(drawing.to_string(), "_Mimsyo界world");

        let drawing = row.draw(&draw_row, 2, BrokenCharacterFillBehavior::Char('_'));
        assert_eq!(drawing.to_string(), "世Mimsy界world");

        let drawing = row.draw(&draw_row, 3, BrokenCharacterFillBehavior::Char('_'));
        assert_eq!(drawing.to_string(), "世HMimsy_world");

        let drawing = row.draw(&draw_row, 14, BrokenCharacterFillBehavior::Char('_'));
        assert_eq!(drawing.to_string(), "世Hello界worldMimsy");

        let drawing = row.draw(&draw_row, 16, BrokenCharacterFillBehavior::Char('_'));
        assert_eq!(drawing.to_string(), "世Hello界world  Mimsy");

        let drawing = row.draw(&gap_draw_row, 0, BrokenCharacterFillBehavior::Char('_'));
        assert_eq!(drawing.to_string(), "[_H]llo界world");
        let drawing = row.draw(&gap_draw_row, 1, BrokenCharacterFillBehavior::Char('_'));
        assert_eq!(drawing.to_string(), "_[He]lo界world");
        let drawing = row.draw(&gap_draw_row, 2, BrokenCharacterFillBehavior::Char('_'));
        assert_eq!(drawing.to_string(), "世[el]o界world");
        let drawing = row.draw(&gap_draw_row, 6, BrokenCharacterFillBehavior::Char('_'));
        assert_eq!(drawing.to_string(), "世Hell[界]orld");
        let drawing = row.draw(&gap_draw_row, 14, BrokenCharacterFillBehavior::Char('_'));
        assert_eq!(drawing.to_string(), "世Hello界world[  ]");
        let drawing = row.draw(&gap_draw_row, 16, BrokenCharacterFillBehavior::Char('_'));
        assert_eq!(drawing.to_string(), "世Hello界world  [  ]");
    }
}
