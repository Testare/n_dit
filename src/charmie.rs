mod charmie_actor;
mod charmie_def;

use std::borrow::Borrow;
use std::fmt::Display;
use std::ops::AddAssign;

use bevy::reflect::TypeUuid;
use crossterm::style::{ContentStyle, StyledContent};
use itertools::Itertools;
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

#[derive(Debug, Default)]
pub enum ColorSupportLevel {
    #[default]
    TrueColor,
    Ansi256,
    Basic,
    Plain,
}

#[derive(Clone, Debug, Default, PartialEq, TypeUuid)]
#[uuid = "a58d71d0-9e0f-4c6d-a078-c5321756579c"]
pub struct CharacterMapImage {
    rows: Vec<CharmieRow>,
}

#[derive(Clone, Copy, Debug)]
pub enum BrokenCharacterFillBehavior {
    Char(char),
    Gap,
}

impl BrokenCharacterFillBehavior {
    fn to_chr_opt(&self) -> Option<char> {
        match self {
            Self::Char(chr) => Some(*chr),
            _ => None,
        }
    }
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

    /// Mostly useful for debugging
    pub fn to_string(&self) -> String {
        self.rows.iter().map(|row| row.to_string()).join("\n")
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

    pub fn add_effect(&mut self, len: u32, style: &ContentStyle) -> &mut Self {
        if len == 0 {
            return self;
        }
        self.fuse_tail_half_char();
        match self.segments.last_mut() {
            Some(CharmieSegment::Effect {
                len: last_len,
                style: last_style,
            }) if *last_style == *style => {
                *last_len += len;
            },
            _ => {
                self.segments
                    .push(CharmieSegment::Effect { len, style: *style });
            },
        }
        self
    }

    fn apply_effect(&mut self, style: &ContentStyle) -> &mut Self {
        self.segments = self
            .segments
            .iter()
            .map(|segment| segment.with_effect(style))
            .collect();
        self
    }

    pub fn add_gap(&mut self, len: u32) -> &mut Self {
        if len == 0 {
            return self;
        }
        self.fuse_tail_half_char();
        if let Some(CharmieSegment::Empty { len: last_len }) = self.segments.last_mut() {
            *last_len += len;
        } else {
            self.segments.push(CharmieSegment::Empty { len });
        }
        self
    }

    pub fn add_half_char(
        &mut self,
        half_char: char,
        replace_char: Option<char>,
        first_half: bool,
        style: &ContentStyle,
    ) -> &mut Self {
        if half_char.width().unwrap_or_default() < 2 {
            return self;
        }
        if !self.segments.is_empty() {
            if let Some(CharmieSegment::HalfChar {
                half_char: last_half_char,
                replace_char: last_replace_char,
                first_half: true,
                style: last_style,
            }) = self.segments.last().cloned()
            {
                self.segments.pop();
                if half_char == last_half_char && !first_half {
                    self.add_text(half_char.to_string(), &add_styles(&last_style, &style));
                    return self;
                }
                if let Some(last_replace_char) = last_replace_char {
                    self.add_text(last_replace_char.to_string(), &last_style);
                } else {
                    self.add_gap(1);
                }
            }
            if !first_half {
                if let Some(replace_char) = replace_char {
                    self.add_text(replace_char.to_string(), &style);
                } else {
                    self.add_gap(1);
                }
                return self;
            }
        }
        self.segments.push(CharmieSegment::HalfChar {
            half_char,
            replace_char,
            first_half,
            style: *style,
        });
        self
    }

    pub fn add_plain_text<S: Borrow<str>>(&mut self, text: S) -> &mut Self {
        self.add_text(text, &ContentStyle::new());
        self
    }

    pub fn add_text<S: Borrow<str>>(&mut self, text: S, style: &ContentStyle) -> &mut Self {
        if text.borrow().len() == 0 {
            return self;
        }
        self.fuse_tail_half_char();
        match self.segments.last_mut() {
            Some(CharmieSegment::Textual {
                text: last_text,
                style: format,
            }) if *format == *style => last_text.push_str(text.borrow()),
            _ => {
                self.segments.push(CharmieSegment::Textual {
                    text: text.borrow().into(),
                    style: *style,
                });
            },
        }
        self
    }

    pub fn add_char(&mut self, ch: char, style: &ContentStyle) -> &mut Self {
        self.fuse_tail_half_char();
        match self.segments.last_mut() {
            Some(CharmieSegment::Textual {
                text: last_text,
                style: format,
            }) if *format == *style => last_text.push(ch),
            _ => {
                self.segments.push(CharmieSegment::Textual {
                    text: ch.into(),
                    style: *style,
                });
            },
        }
        self
    }

    pub fn add_styled_text<D: Display>(&mut self, styled_content: StyledContent<D>) -> &mut Self {
        self.add_text(styled_content.content().to_string(), styled_content.style());
        self
    }

    fn fuse_tail_half_char(&mut self) -> &mut Self {
        if self.len() > 1
            && self
                .segments
                .last()
                .map(|seg| seg.is_half_char())
                .unwrap_or(true)
        {
            if let Some(CharmieSegment::HalfChar {
                replace_char,
                style,
                ..
            }) = self.segments.pop()
            {
                if let Some(replace_char) = replace_char {
                    self.add_text(replace_char.to_string(), &style);
                } else {
                    self.add_gap(1);
                }
            }
        }
        self
    }

    pub fn of_char(ch: char, style: &ContentStyle) -> Self {
        CharmieRow::new().with_char(ch, style)
    }

    pub fn of_effect(len: u32, style: &ContentStyle) -> Self {
        CharmieRow::new().with_effect(len, style)
    }

    pub fn of_gap(len: u32) -> Self {
        CharmieRow::new().with_gap(len)
    }

    pub fn of_plain_text<S: Borrow<str>>(text: S) -> Self {
        CharmieRow::new().with_plain_text(text)
    }

    pub fn of_styled_text<D: Display>(styled_content: StyledContent<D>) -> Self {
        CharmieRow::new().with_styled_text(styled_content)
    }

    pub fn of_text<S: Borrow<str>>(text: S, style: &ContentStyle) -> Self {
        CharmieRow::new().with_text(text, style)
    }

    pub fn with_char(mut self, ch: char, style: &ContentStyle) -> Self {
        self.add_char(ch, style);
        self
    }

    pub fn with_effect(mut self, len: u32, style: &ContentStyle) -> Self {
        self.add_effect(len, style);
        self
    }

    pub fn with_gap(mut self, len: u32) -> Self {
        self.add_gap(len);
        self
    }

    pub fn with_plain_text<S: Borrow<str>>(mut self, text: S) -> Self {
        self.add_plain_text(text);
        self
    }

    pub fn with_styled_text<D: Display>(mut self, styled_content: StyledContent<D>) -> Self {
        self.add_styled_text(styled_content);
        self
    }

    pub fn with_text<S: Borrow<str>>(mut self, text: S, style: &ContentStyle) -> Self {
        self.add_text(text, style);
        self
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
                CharmieSegment::Effect { style, .. } => {
                    let mut under_clip = self.clip(result_len, segment.len(), bcfb);
                    under_clip.apply_effect(style);
                    // Unfortunately, if you draw an effect partially over a full-width character,
                    // that character will be deleted.
                    // This use case will probably be difficult to design for

                    // Implementation idea: clip_return_remainder()?
                    let clip_len = under_clip.len();
                    result += &under_clip;
                    if clip_len < segment.len() {
                        result.add_effect(segment.len() - clip_len, style);
                    }
                },
                CharmieSegment::Empty { .. } => {
                    let under_clip = self.clip(result_len, segment.len(), bcfb);
                    let clip_len = under_clip.len();
                    result += &under_clip;
                    if clip_len < segment.len() {
                        result.add_gap(segment.len() - clip_len);
                    }
                },
                CharmieSegment::Textual { .. } | CharmieSegment::HalfChar { .. } => {
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
                                CharmieSegment::Effect { style, .. } => {
                                    row += &CharmieSegment::Effect {
                                        len: take_until - skip_start,
                                        style: *style,
                                    }
                                },
                                CharmieSegment::Empty { .. } => {
                                    row += &CharmieSegment::Empty {
                                        len: take_until - skip_start,
                                    }
                                },
                                CharmieSegment::HalfChar { .. } => {
                                    row += segment;
                                },
                                CharmieSegment::Textual { text, style } => {
                                    let mut tail_half_char: Option<char> = None;
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
                                                    row.add_half_char(
                                                        ch,
                                                        bcfb.to_chr_opt(),
                                                        false,
                                                        style,
                                                    );
                                                    return Some(None);
                                                } else if current_idx + 1 == take_until {
                                                    // Full width character sliced in half at the end
                                                    tail_half_char = Some(ch);
                                                    return None;
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
                                    if let Some(tail_half_char) = tail_half_char {
                                        row.add_half_char(
                                            tail_half_char,
                                            bcfb.to_chr_opt(),
                                            true,
                                            style,
                                        );
                                    }
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

/// Should NOT contain newline characters
impl From<&String> for CharmieRow {
    fn from(value: &String) -> Self {
        CharmieRow {
            segments: vec![value.clone().into()],
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
            CharmieSegment::Effect { len, style } => self.add_effect(*len, style),
            CharmieSegment::Empty { len } => self.add_gap(*len),
            CharmieSegment::Textual {
                text,
                style: format,
            } => self.add_text(text.as_str(), format),
            CharmieSegment::HalfChar {
                half_char,
                replace_char,
                first_half,
                style,
            } => self.add_half_char(*half_char, *replace_char, *first_half, style),
        };
    }
}

impl AddAssign<CharmieSegment> for CharmieRow {
    fn add_assign(&mut self, rhs: CharmieSegment) {
        match rhs {
            CharmieSegment::Effect { len, style } => self.add_effect(len, &style),
            CharmieSegment::Empty { len } => self.add_gap(len),
            CharmieSegment::Textual {
                text,
                style: format,
            } => self.add_text(text.as_str(), &format),
            CharmieSegment::HalfChar {
                half_char,
                replace_char,
                first_half,
                style,
            } => self.add_half_char(half_char, replace_char, first_half, &style),
        };
    }
}

/// Charmie Segment - Internal representation of a row segment
/// Cannot use StyledContent because it does not provide mutable access
/// to content
///
#[derive(Debug, Clone, PartialEq)]
enum CharmieSegment {
    Textual {
        text: String,
        style: ContentStyle,
    },
    Empty {
        len: u32,
    },
    Effect {
        len: u32,
        style: ContentStyle,
    },
    HalfChar {
        half_char: char,
        replace_char: Option<char>,
        first_half: bool,
        style: ContentStyle,
    },
}

impl CharmieSegment {
    fn is_half_char(&self) -> bool {
        matches!(self, Self::HalfChar { .. })
    }

    fn with_effect(&self, effect_style: &ContentStyle) -> Self {
        match self {
            Self::Textual { text, style } => Self::Textual {
                text: text.to_string(),
                style: add_styles(style, &effect_style),
            },
            Self::Effect { len, style } => Self::Effect {
                len: *len,
                style: add_styles(style, effect_style),
            },
            Self::Empty { len } => Self::Effect {
                len: *len,
                style: *effect_style,
            },
            Self::HalfChar {
                half_char: chr,
                replace_char: replace_chr,
                first_half,
                style,
            } => Self::HalfChar {
                half_char: *chr,
                first_half: *first_half,
                replace_char: *replace_chr,
                style: add_styles(style, effect_style),
            },
        }
    }

    fn len(&self) -> u32 {
        match self {
            Self::Textual { text, .. } => text.width() as u32,
            Self::Empty { len } => *len,
            Self::Effect { len, .. } => *len,
            Self::HalfChar { .. } => 1u32,
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
            CharmieSegment::Effect { len, style } => {
                let segment = format!("{:lensize$}", "", lensize = *len as usize);
                write!(f, "{}", style.apply(segment))
            },
            CharmieSegment::HalfChar {
                replace_char: replace_chr,
                ..
            } => {
                write!(f, "{}", replace_chr.unwrap_or(' '))
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

// HELPER FUNCTION: Combine styles
fn add_styles(lhs: &ContentStyle, rhs: &ContentStyle) -> ContentStyle {
    ContentStyle {
        foreground_color: rhs.foreground_color.or(lhs.foreground_color),
        background_color: rhs.background_color.or(lhs.background_color),
        attributes: rhs.attributes | lhs.attributes,
        // underline_color
    }
}

#[cfg(test)]
mod tests {
    use crossterm::style::Stylize;
    use test_log::test;

    use super::*;

    #[test]
    fn merging_clipped_sides_of_fullwidth_characters() {
        let row = CharmieRow::of_plain_text("世Hello界world");
        let effect = CharmieRow::of_effect(2, &ContentStyle::new().red());

        let affected = row.draw(&effect, 1, BrokenCharacterFillBehavior::Char('_'));
        let expected = format! {"{}ello界world", "世H".red()};
        assert_eq!(expected, affected.to_string());

        let affected = row.draw(&effect, 6, BrokenCharacterFillBehavior::Char('_'));
        let expected = format! {"世Hell{}world", "o界".red()};
        assert_eq!(expected, affected.to_string());

        let drawing = row.draw(
            &CharmieRow::of_gap(1),
            0,
            BrokenCharacterFillBehavior::Char('_'),
        );
        assert_eq!(drawing.to_string(), "世Hello界world");

        let drawing = row.draw(
            &CharmieRow::of_gap(1),
            7,
            BrokenCharacterFillBehavior::Char('_'),
        );
        assert_eq!(drawing.to_string(), "世Hello界world");

        let drawing = row.draw(
            &CharmieRow::of_gap(2),
            1,
            BrokenCharacterFillBehavior::Char('_'),
        );
        assert_eq!(drawing.to_string(), "世Hello界world");

        let drawing = row.draw(
            &CharmieRow::of_gap(2),
            6,
            BrokenCharacterFillBehavior::Char('_'),
        );
        assert_eq!(drawing.to_string(), "世Hello界world");
    }

    #[test]
    fn test_clipping_charmie_row_with_fullwidth_characters() {
        let row = CharmieRow::of_plain_text("世Hello界world");

        let clipped = row.clip(0, 9, BrokenCharacterFillBehavior::Gap);
        assert_eq!(clipped.to_string(), "世Hello界");

        let clipped = row.clip(1, 7, BrokenCharacterFillBehavior::Char('x'));
        assert_eq!(clipped.to_string(), "xHellox");

        let clipped = row.clip(1, 7, BrokenCharacterFillBehavior::Gap);
        assert_eq!(clipped.to_string(), " Hello ");
    }

    #[test]
    fn test_clipping_charmie_row_with_two_segments() {
        let row = CharmieRow::from("Hello".green()).with_plain_text("There!");
        let mut expected = "Hello".green().to_string();
        expected.push_str("There!");
        assert_eq!(row.to_string(), expected);

        let row = CharmieRow::from("Hello".green()).with_plain_text("There!");
        let clipped = row.clip(0, 10, BrokenCharacterFillBehavior::Gap);
        let mut expected = "Hello".green().to_string();
        expected.push_str("There");
        assert_eq!(clipped.to_string(), expected);

        let row = CharmieRow::from("Hello".green()).with_plain_text("There!");
        let clipped = row.clip(6, 4, BrokenCharacterFillBehavior::Gap);
        assert_eq!(clipped.to_string(), "here");
    }

    #[test]
    fn test_draw_effect() {
        let row = CharmieRow::of_plain_text("世Hello界world");
        let effect = CharmieRow::new().with_effect(2, &ContentStyle::new().red());

        let affected = row.draw(&effect, 0, BrokenCharacterFillBehavior::Char('_'));
        let expected = format! {"{}Hello界world", "世".red()};
        assert_eq!(expected, affected.to_string());

        let affected = row.draw(&effect, 2, BrokenCharacterFillBehavior::Char('_'));
        let expected = format! {"世{}llo界world", "He".red()};
        assert_eq!(expected, affected.to_string());

        let affected = row.draw(&effect, 15, BrokenCharacterFillBehavior::Char('_'));
        let expected = format! {"世Hello界world {}", "  ".red()};
        assert_eq!(expected, affected.to_string());
    }

    #[test]
    fn test_draw_charmie_row() {
        let bcr = BrokenCharacterFillBehavior::Char('_');
        let row = CharmieRow::of_plain_text("世Hello界world");
        let draw_row = CharmieRow::of_plain_text("Mimsy");
        let gap_draw_row = CharmieRow::of_plain_text("[")
            .with_gap(2)
            .with_plain_text("]");

        let drawing = row.draw(&draw_row, 0, bcr);
        assert_eq!(drawing.to_string(), "Mimsylo界world");

        let drawing = row.draw(&draw_row, 1, bcr);
        assert_eq!(drawing.to_string(), "_Mimsyo界world");

        let drawing = row.draw(&draw_row, 2, bcr);
        assert_eq!(drawing.to_string(), "世Mimsy界world");

        let drawing = row.draw(&draw_row, 3, bcr);
        assert_eq!(drawing.to_string(), "世HMimsy_world");

        let drawing = row.draw(&draw_row, 14, bcr);
        assert_eq!(drawing.to_string(), "世Hello界worldMimsy");

        let drawing = row.draw(&draw_row, 16, bcr);
        assert_eq!(drawing.to_string(), "世Hello界world  Mimsy");

        let drawing = row.draw(&gap_draw_row, 0, bcr);
        assert_eq!(drawing.to_string(), "[_H]llo界world");
        let drawing = row.draw(&gap_draw_row, 1, bcr);
        assert_eq!(drawing.to_string(), "_[He]lo界world");
        let drawing = row.draw(&gap_draw_row, 2, bcr);
        assert_eq!(drawing.to_string(), "世[el]o界world");
        let drawing = row.draw(&gap_draw_row, 6, bcr);
        assert_eq!(drawing.to_string(), "世Hell[界]orld");
        let drawing = row.draw(&gap_draw_row, 14, bcr);
        assert_eq!(drawing.to_string(), "世Hello界world[  ]");
        let drawing = row.draw(&gap_draw_row, 16, bcr);
        assert_eq!(drawing.to_string(), "世Hello界world  [  ]");

        let drawing = row.draw(
            &CharmieRow::of_gap(2),
            0,
            BrokenCharacterFillBehavior::Char('_'),
        );
        assert_eq!(drawing.to_string(), "世Hello界world");
    }
}
