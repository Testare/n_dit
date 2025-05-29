use std::cmp::Ordering;
use std::sync::Arc;

use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

use super::{color::CharmiColor, style::CharmiStyle};
use crate::{CharCell, CharmiFill, CharmiImage, SanitizedText};

#[derive(Clone, Debug)]
enum CharmiBuilderMode {
    FixedSize {
        image: CharmiImage,
    },
    Dynamic {
        width: Option<usize>,
        lines: Vec<Vec<CharCell>>,
    },
}

impl Default for CharmiBuilderMode {
    fn default() -> Self {
        Self::Dynamic {
            width: None,
            lines: vec![Vec::new()],
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct CharmiBuilderSettings {
    style: CharmiStyle,
    cursor: (usize, usize),
    empty_char: Option<char>,
    split_char: (u32, u32),
    wrap_spacing: bool,
    wrap_hyphenate: bool,
    fill: CharCell,
}

impl Default for CharmiBuilderSettings {
    fn default() -> Self {
        Self {
            style: CharmiStyle {
                fg: 0x0F, // TODO BEFOREMERGE default should be NC to match definition API
                bg: 0x00,
                attr: 0x00,
            },
            cursor: (0, 0),
            empty_char: None,
            split_char: (' ' as u32, ' ' as u32),
            wrap_spacing: true,
            wrap_hyphenate: false,
            fill: CharCell::GAP,
        }
    }
}

#[derive(Clone, Debug)]
pub struct CharmiBuilder {
    mode: CharmiBuilderMode,
    settings: CharmiBuilderSettings,
}

impl CharmiBuilder {
    const FILL_INDICATOR: u32 = 0xFDD0;
    const FILL_STYLED_INDICATOR: u32 = 0xFDD1; // New constant
    const FILL_CELL: CharCell = CharCell {
        ch: Self::FILL_INDICATOR,
        fg: CharmiImage::NO_COLOR,
        bg: CharmiImage::NO_COLOR,
        attr: 0,
    };

    pub fn fixed_size(width: u32, height: u32) -> Self {
        let settings: CharmiBuilderSettings = Default::default();
        Self {
            mode: CharmiBuilderMode::FixedSize {
                image: CharmiImage::new_fill(width, height, Self::FILL_CELL),
            },
            settings,
        }
    }

    pub fn fixed_width(width: u32) -> Self {
        let settings: CharmiBuilderSettings = Default::default();
        Self {
            mode: CharmiBuilderMode::Dynamic {
                width: Some(width as usize),
                lines: vec![],
            },
            settings,
        }
    }

    pub fn dynamic() -> Self {
        let settings: CharmiBuilderSettings = Default::default();
        Self {
            mode: CharmiBuilderMode::Dynamic {
                width: None,
                lines: vec![],
            },
            settings,
        }
    }

    pub fn settings(&self) -> &CharmiBuilderSettings {
        &self.settings
    }

    /// Fixed-size by default
    pub fn edit(image: CharmiImage) -> Self {
        Self {
            mode: CharmiBuilderMode::FixedSize { image },
            settings: Default::default(),
        }
    }

    pub fn as_fixed_size(&mut self, width: u32, height: u32) -> &mut Self {
        self.clip(0, 0, width, height)
    }

    pub fn as_fixed_width(&mut self, new_width: u32) -> &mut Self {
        let new_mode = match &mut self.mode {
            CharmiBuilderMode::Dynamic { width, lines } => {
                for line in lines.iter_mut() {
                    // Make sure each line is properly truncated and cracked
                    let line_len = line.len().min(new_width as usize);
                    if line_len < line.len() && line[line_len].ch == CharmiImage::SUPPRESSED_CHAR {
                        line[line_len - 1].ch = line[line_len].bg;
                    }
                    line.truncate(line_len);
                }
                *width = Some(new_width as usize);
                return self;
            },
            CharmiBuilderMode::FixedSize { image } => {
                let cells = std::mem::take(&mut image.cells);
                CharmiBuilderMode::Dynamic {
                    width: Some(new_width as usize),
                    lines: cells
                        .chunks(image.width as usize)
                        .map(|src_line| {
                            // Make sure each line is properly truncated and cracked
                            let line_len = src_line.len().min(new_width as usize);
                            let mut dst_line = Vec::from(&src_line[..line_len]);
                            if line_len < src_line.len()
                                && src_line[line_len].ch == CharmiImage::SUPPRESSED_CHAR
                            {
                                dst_line[line_len - 1].ch = src_line[line_len].bg;
                            }
                            dst_line
                        })
                        .collect(),
                }
            },
        };
        self.mode = new_mode;
        self
    }

    pub fn as_dynamic_size(&mut self) -> &mut Self {
        let new_mode = match &mut self.mode {
            CharmiBuilderMode::Dynamic { width, .. } => {
                *width = None;
                return self;
            },
            CharmiBuilderMode::FixedSize { image } => {
                let cells = std::mem::take(&mut image.cells);
                CharmiBuilderMode::Dynamic {
                    width: None,
                    lines: cells.chunks(image.width as usize).map(Vec::from).collect(),
                }
            },
        };
        self.mode = new_mode;
        self
    }

    /// Draws another image at the cursor.
    pub fn draw(&mut self, src_image: &CharmiImage) -> &mut Self {
        if src_image.width == 0 {
            return self;
        }

        let cursor = self.settings.cursor;
        let image_lines = src_image.cells.chunks(src_image.width as usize);
        let max_lines = if let CharmiBuilderMode::FixedSize { image } = &self.mode {
            (image.height as usize).saturating_sub(cursor.1)
        } else {
            usize::MAX
        };
        for (i, line) in image_lines.enumerate().take(max_lines) {
            if i != 0 {
                self.next_line();
                self.skip_cells(cursor.0);
            }
            self.add_sanitized_text(line, false);
        }
        self
    }

    pub fn clip(&mut self, x: i32, y: i32, width: u32, height: u32) -> &mut Self {
        self.mode = CharmiBuilderMode::FixedSize {
            image: self
                .build_internal(false)
                .clip(x, y, width, height, Self::FILL_CELL),
        };
        self
    }

    /// Different from [`clip`] when clip dimensions are outside of the original image, [`clip`]
    /// will resize it to proper size, whereas [`clip_intersect`] will just be the portion of the
    /// original image within the clip.
    ///
    /// For example:
    ///
    /// ```text
    /// charmi = ABC
    ///          DEF
    /// charmi.clip(-1, 1, 5, 1, None)  => "[gap]ABC[gap]"
    /// charmi.clip_intersect(-1, 1, 5, 1) => "ABC"
    /// ```
    pub fn clip_intersect(&mut self, x: u32, y: u32, width: u32, height: u32) -> &mut Self {
        self.mode = CharmiBuilderMode::FixedSize {
            image: self
                .build_internal(false)
                .clip_intersect(x, y, width, height),
        };
        self
    }

    pub fn set_cursor(&mut self, x: usize, y: usize) -> &mut Self {
        let (next_x, next_y) = match &self.mode {
            CharmiBuilderMode::Dynamic { width, .. } => {
                let x = width.map(|w| w.min(x)).unwrap_or(x);
                (x, y)
            },
            CharmiBuilderMode::FixedSize { image } => {
                (x.min(image.width as usize), y.min(image.height as usize))
            },
        };
        self.settings.cursor.1 = next_y;
        self.skip_cells(next_x.saturating_sub(self.settings.cursor.0));
        self.settings.cursor.0 = next_x;
        self
    }

    pub fn next_line(&mut self) -> &mut Self {
        self.skip_lines(1);
        self
    }

    /// Skips the builder down `n` lines.
    ///
    /// Note that `skip_lines(1) is the same as [`next_line()`]. If you want 1 blank row, you'd
    /// have to do `skip_lines(2)`.
    ///
    ///
    ///
    pub fn skip_lines(&mut self, n: usize) -> &mut Self {
        if n == 0 {
            return self;
        }
        let Self { mode, settings } = self;
        let last_pos = settings.cursor;
        match mode {
            CharmiBuilderMode::Dynamic { lines, .. } => {
                settings.cursor = (0, last_pos.1 + n);
                let append_n = (settings.cursor.1 + 1).saturating_sub(lines.len());
                lines.extend(std::iter::repeat_n(Vec::default(), append_n));
            },
            CharmiBuilderMode::FixedSize { image } => {
                settings.cursor = (0, (last_pos.1 + n).min(image.height as usize));
            },
        }
        self
    }

    pub fn skip_cells(&mut self, n: usize) -> &mut Self {
        self.line_check();
        let Self { settings, mode } = self;
        match mode {
            CharmiBuilderMode::Dynamic { width, lines, .. } => {
                settings.cursor.0 = (settings.cursor.0 + n).min(width.unwrap_or(usize::MAX));
                let line = &mut lines[settings.cursor.1];
                if line.len() < settings.cursor.0 {
                    line.extend(std::iter::repeat_n(
                        Self::FILL_CELL,
                        settings.cursor.0 - line.len(),
                    ));
                }
            },
            CharmiBuilderMode::FixedSize { image } => {
                settings.cursor.0 = (settings.cursor.0 + n).min(image.width as usize);
            },
        }
        self
    }

    pub fn add_text(&mut self, text: &str) -> &mut Self {
        let text = self.sanitize_text(text);
        self.add_sanitized_text(&text, false)
    }

    pub fn add_line(&mut self, text: &str) -> &mut Self {
        self.add_text(text);
        // We don't use self.next_line(), so that we don't expand image unless we desire to
        self.soft_newline();
        self
    }

    /// Adds a gap N cells long, or until end of row for fixed width/fixed size.
    ///
    /// Different from skip_chars in cases when editting an image or when image
    /// has a fill in that this will replace existing values with empty/passthrough
    /// gaps, where [`skip_chars`] does not change the image (On new images, this
    /// will be whatever the configured fill is).
    ///
    /// When editting, this can be thought of as "erasing" n chars
    pub fn add_gap(&mut self, n: usize) -> &mut Self {
        let range = self.crack_range(n);
        for cell in range {
            *cell = CharCell::GAP;
        }
        self
    }

    /// Adds an effect N cells long, or until the end of the row for fixed width/fixed size.
    ///
    /// This is different from `add_gap` because the gap will have the fg/bg set for the builder.
    /// It is different from vanilla `add_text(' ')` because chars under the effect will pass-through,
    ///
    /// This is different from "apply_style", because it will replace characters with a gap,
    /// whereas apply_effect keeps the characters the same.
    pub fn add_style_effect(&mut self, n: usize) -> &mut Self {
        let effect_cell = CharCell {
            ch: 0,
            fg: self.settings.style.fg,
            bg: self.settings.style.bg,
            attr: 0,
        };
        let range = self.crack_range(n);
        for cell in range {
            *cell = effect_cell;
        }
        self
    }

    pub fn apply_style(&mut self, n: usize) -> &mut Self {
        let style_cell = self.settings.style;
        let range = self.crack_range(n);
        for cell in range {
            let ch = if cell.ch == Self::FILL_INDICATOR {
                Self::FILL_STYLED_INDICATOR
            } else {
                cell.ch
            };
            *cell = style_cell.of_ch(ch);
        }
        self
    }

    pub fn add_text_wrap_chars(&mut self, text: &str) -> &mut Self {
        self.line_check();
        let sanitized_text = self.sanitize_text(text);
        let text_len = sanitized_text.len();
        let mut i = 0;
        let hyphen_width = if self.settings.wrap_hyphenate { 1 } else { 0 };
        match self.max_width() {
            Some(0) => {},
            Some(1) => {
                for i in 0..text_len {
                    if sanitized_text[i].ch == CharmiImage::SUPPRESSED_CHAR {
                        self.add_sanitized_text(
                            &[CharCell {
                                ch: sanitized_text[i].fg,
                                ..sanitized_text[i - 1]
                            }],
                            false,
                        );
                    } else {
                        // We add a range of 2 if we can so the left cracking is handled for us
                        let next_i = (i + 2).min(text_len);
                        self.add_sanitized_text(&sanitized_text[i..next_i], false);
                    }
                    self.soft_newline();
                }
            },
            _ => {
                while i < text_len {
                    let mut next_i = i + self.remaining_space_on_line();
                    match next_i.cmp(&text_len) {
                        Ordering::Less => {
                            next_i -= hyphen_width; // We'll need space for the hyphen if it exists
                            if sanitized_text[next_i].ch == CharmiImage::SUPPRESSED_CHAR {
                                next_i -= 1;
                            }
                            self.add_sanitized_text(&sanitized_text[i..next_i], false);
                            if self.settings.wrap_hyphenate {
                                self.add_text("-");
                            }
                            self.next_line();
                        },
                        Ordering::Equal => {
                            self.add_sanitized_text(&sanitized_text[i..], false);
                            self.soft_newline();
                        },
                        Ordering::Greater => {
                            next_i = text_len;
                            self.add_sanitized_text(&sanitized_text[i..], false);
                        },
                    }
                    i = next_i;
                }
            },
        }

        self
    }

    /// Adds a section of text wholesale. If the section of text won't fit, will add
    /// it to the next line. If it wouldn't fit their either, it instead follows the
    /// logic of [`add_text_wrap_chars`], since the expected use case for this
    /// method is to draw text and that seems a logical fallback. If you have a use
    /// case where this fallback logic is undesirable, let us know.
    pub fn add_text_wrap_line(&mut self, text: &str) -> &mut Self {
        // It is assumed that text.width() will yield the same result as sanitized_text.len()
        if text.width() > self.max_width().unwrap_or(usize::MAX) {
            // Word is too big for a single line, must break up word and wrap the characters.
            self.add_text_wrap_chars(text)
        } else {
            if text.len() > self.remaining_space_on_line() {
                self.next_line();
            }
            self.add_text(text)
        }
    }

    /// If any word is too wide for the image by itself, we will follow the logic of
    /// [`add_text_wrap_chars`] for that word, since the expected use case for this
    /// method is to draw text and that seems a logical fallback. If you have a use
    /// case where this fallback logic is undesirable, let us know.
    pub fn add_text_wrap_words(&mut self, words: &[&str]) -> &mut Self {
        let max_width = self.max_width().unwrap_or(usize::MAX);
        let space = [CharCell {
            ch: ' ' as u32,
            fg: self.settings.style.fg,
            bg: self.settings.style.bg,
            attr: 0,
        }];
        for (i, word) in words.iter().enumerate() {
            // It is assumed that text.width() will yield the same result as sanitized_text.len()
            let required_space = if i != 0 && self.settings.wrap_spacing {
                1
            } else {
                0
            };
            if word.width() > max_width {
                if required_space == 1 && self.remaining_space_on_line() > 0 {
                    self.add_sanitized_text(&space, false);
                }
                // Word is too big for a single line, must break up word and wrap the characters.
                self.add_text_wrap_chars(word);
                continue;
            }
            if (required_space + word.len()) <= self.remaining_space_on_line() {
                if required_space == 1 {
                    self.add_sanitized_text(&space, false);
                }
                self.add_text(word);
            } else {
                self.next_line().add_text(word);
            }
        }
        self
    }

    pub fn wrap_with_spacing(&mut self) -> &mut Self {
        self.settings.wrap_spacing = true;
        self
    }

    pub fn wrap_without_spacing(&mut self) -> &mut Self {
        self.settings.wrap_spacing = false;
        self
    }

    /// Sets the fill. If the fill is just a character, will use CURRENT fg/bg.
    /// Fill is applied at build step.
    pub fn with_fill<F: CharmiFill>(&mut self, fill: F) -> &mut Self {
        self.settings.fill = fill.as_fill(&self.settings.style.of_ch(0));
        self
    }

    /// Sets the character used to represent gaps in the text methods
    pub fn gap_char(&mut self, ch: char) -> &mut Self {
        assert_eq!(
            Some(1),
            ch.width(),
            "The width of the character used for the empty character must be 1",
        );
        self.settings.empty_char = Some(ch);
        self
    }

    /// Reset builder so that there is no character that signifies a gap.
    pub fn no_gap_char(&mut self) -> &mut Self {
        self.settings.empty_char = None;
        self
    }

    pub fn split_char(&mut self, ch: char) -> &mut Self {
        // Should I add precondition to check that ch.width() == Some(1)?
        self.settings.split_char = (ch as u32, ch as u32);
        self
    }

    pub fn split_chars(&mut self, left: char, right: char) -> &mut Self {
        self.settings.split_char = (left as u32, right as u32);
        self
    }

    pub fn split_char_gap(&mut self) -> &mut Self {
        self.settings.split_char = (0, 0);
        self
    }

    pub fn split_char_space(&mut self) -> &mut Self {
        self.split_char(' ')
    }

    pub fn fg(&mut self, color: impl CharmiColor) -> &mut Self {
        self.settings.style.fg = color.as_color_u32();
        self
    }

    pub fn no_fg(&mut self) -> &mut Self {
        self.settings.style.fg = CharmiImage::NO_COLOR;
        self
    }

    pub fn bg(&mut self, color: impl CharmiColor) -> &mut Self {
        self.settings.style.bg = color.as_color_u32();
        self
    }

    pub fn no_bg(&mut self) -> &mut Self {
        self.settings.style.bg = CharmiImage::NO_COLOR;
        self
    }

    pub fn style<S: Into<CharmiStyle>>(&mut self, style: S) -> &mut Self {
        self.settings.style = style.into();
        self
    }

    pub fn wrap_hyphenate(&mut self, wrap_hyphenate: bool) -> &mut Self {
        self.settings.wrap_hyphenate = wrap_hyphenate;
        self
    }

    pub fn build(&mut self) -> CharmiImage {
        self.build_internal(true)
    }

    /// Functionally identical to [`build`], but this name might be preferable when editting
    pub fn apply(&mut self) -> CharmiImage {
        self.build()
    }

    // HELPER FUNCTIONS

    /// Internal build function - Allows us to control when fill is applied, which is important for clipping.
    fn build_internal(&mut self, apply_fill: bool) -> CharmiImage {
        let mode = std::mem::take(&mut self.mode);
        let fill = if apply_fill {
            self.settings.fill
        } else {
            Self::FILL_CELL
        };

        match mode {
            CharmiBuilderMode::Dynamic { width, lines } => {
                let width =
                    width.unwrap_or_else(|| lines.iter().map(|line| line.len()).max().unwrap_or(0));
                let height = lines.len();
                if width == 0 || height == 0 {
                    return CharmiImage::default();
                }
                let cells = lines
                    .into_iter()
                    .flat_map(|mut line| {
                        if apply_fill {
                            for cell in line.iter_mut() {
                                match cell.ch {
                                    Self::FILL_INDICATOR => *cell = fill,
                                    Self::FILL_STYLED_INDICATOR => cell.ch = fill.ch,
                                    _ => {},
                                }
                            }
                        }
                        let line_len = line.len();
                        line.into_iter()
                            .chain(std::iter::repeat_n(fill, width - line_len))
                    })
                    .collect();
                unsafe { CharmiImage::from_raw_cells(width as u32, height as u32, cells) }
            },
            CharmiBuilderMode::FixedSize { mut image } => {
                if apply_fill {
                    let cells = Arc::make_mut(&mut image.cells);
                    for cell in cells.iter_mut() {
                        match cell.ch {
                            Self::FILL_INDICATOR => *cell = fill,
                            Self::FILL_STYLED_INDICATOR => cell.ch = fill.ch,
                            _ => {},
                        }
                    }
                }
                image
            },
        }
    }

    /// Sanitizes text
    fn sanitize_text(&self, text: &str) -> Vec<CharCell> {
        let CharmiBuilderSettings {
            style: cell,
            split_char,
            empty_char,
            ..
        } = self.settings;
        SanitizedText::from_text_full(text, Some(cell), Some(split_char), empty_char).unwrap()
    }

    /// Bulk of adding text logic, after text has been sanitized.
    fn add_sanitized_text(
        &mut self,
        src_range: &[CharCell],
        gaps_overwrite_chars: bool,
    ) -> &mut Self {
        // TODO
        // FUTURE WORK: Optimize this method so that if we never use a method like "set_cursor" or "edit()", we
        // know that we don't have to worry about what we're drawing over, so we can just add the text simply
        self.line_check();
        if src_range.is_empty() {
            // Keep it simple, keeps us from worrying about edge-case related bugs
            return self;
        }
        let Self { mode, settings } = self;
        let pos = &mut settings.cursor;
        let next_src_i_start = if pos.0 == 0 { 1 } else { 0 };

        let (dst_range, max_width) = match mode {
            CharmiBuilderMode::FixedSize { image } => {
                if pos.0 as u32 >= image.width || pos.1 as u32 >= image.height {
                    return self;
                }
                let row_start = (image.width as usize) * pos.1;
                let edit_start = pos.0.saturating_sub(1) + row_start;
                let row_end = row_start + (image.width as usize);
                (
                    &mut Arc::make_mut(&mut image.cells)[edit_start..row_end],
                    image.width as usize,
                )
            },
            CharmiBuilderMode::Dynamic { width, lines } => {
                if width.map(|w| pos.0 >= w).unwrap_or(false) {
                    return self;
                }
                let start_pos = pos.0.saturating_sub(1);
                let line = &mut lines[pos.1];
                let append_n = (src_range.len() + pos.0)
                    .min(width.unwrap_or(usize::MAX))
                    .saturating_sub(line.len());

                line.extend(std::iter::repeat_n(Self::FILL_CELL, append_n));
                let width = line.len();
                (&mut line[start_pos..], width)
            },
        };

        let mut last_dst_i = 0;
        let mut src_next: Option<&CharCell> = None;

        let mut replacement_cells: Vec<CharCell> = (0usize..dst_range.len())
            .zip((next_src_i_start as usize)..(src_range.len() + 2))
            .map(|(dst_i, src_next_i)| {
                last_dst_i = dst_i;
                src_next = src_range.get(src_next_i);
                let src_i = src_next_i.checked_sub(1);
                let src = src_i.and_then(|i| src_range.get(i));
                if let Some(src) = src {
                    if src.ch != 0 || gaps_overwrite_chars {
                        return *src;
                    }
                }
                let dst = dst_range[dst_i];
                let result = if dst.ch == CharmiImage::SUPPRESSED_CHAR {
                    let src_prev_is_passthrough = src_next_i
                        .checked_sub(2)
                        .map(|i| src_range[i].ch == 0 && !gaps_overwrite_chars)
                        .unwrap_or(true);
                    if src_prev_is_passthrough {
                        dst
                    } else {
                        // Crack forward
                        // src_prev will always be passtrhough when dst == 0, so dst_i should be safe
                        CharCell {
                            ch: dst.fg,
                            ..dst_range[dst_i - 1]
                        }
                    }
                } else if matches!(
                    dst_range.get(dst_i + 1),
                    Some(CharCell {
                        ch: CharmiImage::SUPPRESSED_CHAR,
                        ..
                    })
                ) {
                    let src_next_is_passthrough = src_range
                        .get(src_next_i)
                        .map(|src_next| src_next.ch == 0 && !gaps_overwrite_chars)
                        .unwrap_or(true);
                    if src_next_is_passthrough {
                        dst
                    } else {
                        // Crack back
                        // We verified previously replacment_range[dst_i +1] is Some
                        CharCell {
                            ch: dst_range[dst_i + 1].bg,
                            ..dst
                        }
                    }
                } else {
                    dst
                };
                src.map(|src| src.draw_to(&result)).unwrap_or(result)
            })
            .collect();

        if let Some(src_next) = src_next {
            if src_next.ch == CharmiImage::SUPPRESSED_CHAR {
                // The last character can't have been passthrough: SUPPRESSED_CHAR cannot follow GAP
                // and it can't be the first character in a row
                // Backcrack
                replacement_cells
                    .last_mut()
                    .expect("text should be at least one cell big")
                    .ch = src_next.bg;
            }
        }

        pos.0 = (pos.0 + src_range.len()).min(max_width);
        dst_range[0..replacement_cells.len()].copy_from_slice(&replacement_cells);
        self
    }

    /// Returns a mutable slice of cells, handling:
    /// - Range extension for dynamic mode
    /// - Character cracking at range boundaries
    /// - Width limits
    ///
    /// Returns the actual range of cells that can be modified
    fn crack_range(&mut self, len: usize) -> &mut [CharCell] {
        self.line_check();
        let Self { settings, mode } = self;
        match mode {
            CharmiBuilderMode::Dynamic { width, lines } => {
                if width.map(|w| settings.cursor.0 >= w).unwrap_or(false) {
                    return &mut [];
                }
                let line = &mut lines[settings.cursor.1];
                let start = settings.cursor.0;
                let end = (start + len).min(width.unwrap_or(usize::MAX));

                // Extend line if needed
                if end > line.len() {
                    line.extend(std::iter::repeat_n(Self::FILL_CELL, end - line.len()));
                }

                // Handle cracking at start
                if start > 0 && line[start].ch == CharmiImage::SUPPRESSED_CHAR {
                    line[start].ch = line[start].fg;
                    line[start].fg = line[start - 1].fg;
                    line[start].bg = line[start - 1].bg;
                }

                // Handle cracking at end
                if end < line.len() && line[end].ch == CharmiImage::SUPPRESSED_CHAR {
                    line[end - 1].ch = line[end].bg;
                }

                settings.cursor.0 = end;
                &mut line[start..end]
            },
            CharmiBuilderMode::FixedSize { image } => {
                if settings.cursor.0 as u32 >= image.width
                    || settings.cursor.1 as u32 >= image.height
                {
                    return &mut [];
                }
                let cells = Arc::make_mut(&mut image.cells);
                let row_start = (image.width as usize) * settings.cursor.1;
                let start = settings.cursor.0 + row_start;
                let end = (start + len).min(row_start + image.width as usize);

                // Handle cracking at start
                if start > row_start && cells[start].ch == CharmiImage::SUPPRESSED_CHAR {
                    // Crack left
                    cells[start].ch = cells[start].fg;
                    cells[start].fg = cells[start - 1].fg;
                    cells[start].bg = cells[start - 1].bg;
                }

                // Handle cracking at end
                if end < (row_start + image.width as usize)
                    && cells[end].ch == CharmiImage::SUPPRESSED_CHAR
                {
                    cells[end - 1].ch = cells[end].bg;
                }

                settings.cursor.0 = (settings.cursor.0 + len).min(image.width as usize);
                &mut cells[start..end]
            },
        }
    }

    fn max_width(&self) -> Option<usize> {
        match &self.mode {
            CharmiBuilderMode::Dynamic { width, .. } => *width,
            CharmiBuilderMode::FixedSize { image } => Some(image.width as usize),
        }
    }

    fn remaining_space_on_line(&self) -> usize {
        match &self.mode {
            CharmiBuilderMode::FixedSize { image } => image.width as usize - self.settings.cursor.0,
            CharmiBuilderMode::Dynamic { width, .. } => width
                .map(|w| w - self.settings.cursor.0)
                .unwrap_or(usize::MAX),
        }
    }

    /// Utility function to append lines to dynamic builders as needed.
    fn line_check(&mut self) {
        if let Self {
            mode: CharmiBuilderMode::Dynamic { lines, .. },
            settings,
        } = self
        {
            if lines.len() <= settings.cursor.1 {
                let append_n = (settings.cursor.1 + 1).saturating_sub(lines.len());
                lines.extend(std::iter::repeat_n(Vec::default(), append_n));
            }
        }
    }

    fn soft_newline(&mut self) {
        self.settings.cursor = (0, self.settings.cursor.1 + 1);
    }
}

impl Default for CharmiBuilder {
    fn default() -> Self {
        Self::dynamic()
    }
}

#[cfg(test)]
mod test {
    use test_util::*;

    use super::*;

    mod test_util {
        use super::*;

        pub const SC: u32 = CharmiImage::SUPPRESSED_CHAR;
        pub const NC: u32 = CharmiImage::NO_COLOR;

        pub fn charmi_line(cells: &[u32]) -> CharmiImage {
            unsafe {
                // SAFETY to be verified manually by test cases
                CharmiImage::from_raw_u32((cells.len() / 4) as u32, 1, cells)
            }
        }

        /// Helper method to create a charmi image with a height of 2
        pub fn charmi_double_line(cells: &[u32]) -> CharmiImage {
            unsafe {
                // SAFETY to be verified manually by test cases
                CharmiImage::from_raw_u32((cells.len() / 8) as u32, 2, cells)
            }
        }

        #[test]
        fn test_util_charmi_line() {
            #[rustfmt::skip]
            let actual = charmi_line(&[
                '<' as u32, 1, 6, 0,
                '-' as u32, 2, 6, 0,
                '-' as u32, 3, 5, 0,
                '-' as u32, 2, 6, 1,
                '>' as u32, 1, 6, 0,
            ]);

            let expected = CharmiImage {
                width: 5,
                height: 1,
                cells: Arc::new([
                    CharCell {
                        ch: '<' as u32,
                        fg: 1,
                        bg: 6,
                        attr: 0,
                    },
                    CharCell {
                        ch: '-' as u32,
                        fg: 2,
                        bg: 6,
                        attr: 0,
                    },
                    CharCell {
                        ch: '-' as u32,
                        fg: 3,
                        bg: 5,
                        attr: 0,
                    },
                    CharCell {
                        ch: '-' as u32,
                        fg: 2,
                        bg: 6,
                        attr: 1,
                    },
                    CharCell {
                        ch: '>' as u32,
                        fg: 1,
                        bg: 6,
                        attr: 0,
                    },
                ]),
            };

            assert_eq!(expected, actual);
        }

        #[test]
        fn test_util_charmi_double_line() {
            #[rustfmt::skip]
            let actual = charmi_double_line(&[
                '<' as u32, 1, 6, 0,
                '-' as u32, 2, 6, 1,
                '>' as u32, 1, 6, 0,
                // ---
                '>' as u32, 1, 6, 0,
                '-' as u32, 2, 6, 1,
                '<' as u32, 1, 6, 0,

            ]);

            let expected = CharmiImage {
                width: 3,
                height: 2,
                cells: Arc::new([
                    CharCell {
                        ch: '<' as u32,
                        fg: 1,
                        bg: 6,
                        attr: 0,
                    },
                    CharCell {
                        ch: '-' as u32,
                        fg: 2,
                        bg: 6,
                        attr: 1,
                    },
                    CharCell {
                        ch: '>' as u32,
                        fg: 1,
                        bg: 6,
                        attr: 0,
                    },
                    CharCell {
                        ch: '>' as u32,
                        fg: 1,
                        bg: 6,
                        attr: 0,
                    },
                    CharCell {
                        ch: '-' as u32,
                        fg: 2,
                        bg: 6,
                        attr: 1,
                    },
                    CharCell {
                        ch: '<' as u32,
                        fg: 1,
                        bg: 6,
                        attr: 0,
                    },
                ]),
            };

            assert_eq!(expected, actual);
        }
    }

    mod conversion {
        use super::*;

        #[test]
        fn as_fixed_size() {
            #[rustfmt::skip]
            let expected = charmi_double_line(&[
                'A' as u32, 1, 6, 0,
                'B' as u32, 1, 6, 0,
                0         , NC, NC, 0,
                0         , NC, NC, 0,
                0         , NC, NC, 0,
                0         , NC, NC, 0,
                0         , NC, NC, 0,
                0         , NC, NC, 0,
            ]);

            let actual = CharmiBuilder::dynamic()
                .fg(1)
                .bg(6)
                .add_text("AB")
                .as_fixed_size(4, 2)
                .build();
            println!("Case 1: Convert from dynamic mode");
            assert_eq!(expected, actual);

            let actual = CharmiBuilder::fixed_width(2)
                .fg(1)
                .bg(6)
                .add_text("AB")
                .as_fixed_size(4, 2)
                .build();
            println!("Case 2: Convert from fixed width mode");
            assert_eq!(expected, actual);

            let actual = CharmiBuilder::fixed_width(2)
                .fg(1)
                .bg(6)
                .add_text("AB")
                .as_fixed_size(4, 2)
                .build();
            println!("Case 3: Convert to a larger size");
            assert_eq!(expected, actual);

            #[rustfmt::skip]
            let expected = charmi_line(&[
                'A' as u32, 1, 6, 0,
                '<' as u32, 1, 6, 0, // Test cracking works
            ]);
            let actual = CharmiBuilder::dynamic()
                .fg(1)
                .bg(6)
                .split_chars('<', '>')
                .add_text("AバCDEF")
                .as_fixed_size(2, 1)
                .build();
            println!("Case 4: Test truncating an image and cracking");
            assert_eq!(expected, actual);
        }

        #[test]
        fn as_fixed_width() {
            // Should crack バ at end of new width
            #[rustfmt::skip]
            let expected = charmi_line(&[
                'バ' as u32,  1,  6, 0,
                SC         , 32, 32, 0,
                32         ,  1,  6, 0,  // Cracked character
            ]);

            let actual = CharmiBuilder::dynamic()
                .fg(1)
                .bg(6)
                .add_text("バカA")
                .build()
                .edit()
                .as_fixed_width(3)
                .build();
            println!("Case 1: Basic cracking");
            assert_eq!(expected, actual);

            // Test converting from Dynamic mode
            let actual = CharmiBuilder::dynamic()
                .fg(1)
                .bg(6)
                .add_text("バカA")
                .as_fixed_width(3)
                .build();
            println!("Case 2: Dynamic mode conversion with cracking");
            assert_eq!(expected, actual);

            // Test no cracking needed when width aligns with character boundaries
            #[rustfmt::skip]
            let expected = charmi_line(&[
                'バ' as u32,  1,  6, 0,
                SC         , 32, 32, 0,
                'カ' as u32,  1,  6, 0,
                SC         , 32, 32, 0,
            ]);

            let actual = CharmiBuilder::dynamic()
                .fg(1)
                .bg(6)
                .add_text("バカA")
                .build()
                .edit()
                .as_fixed_width(4)
                .build();
            println!("Case 3: No cracking needed");
            assert_eq!(expected, actual);

            // Test conversion to larger width with subsequent text addition
            #[rustfmt::skip]
            let expected = charmi_line(&[
                'A' as u32,   1,  6, 0,
                'バ' as u32,  1,  6, 0,
                SC         , 32, 32, 0,
                'B' as u32,   2,  7, 0,
                'C' as u32,   2,  7, 0,
            ]);

            let actual = CharmiBuilder::dynamic()
                .fg(1)
                .bg(6)
                .add_text("Aバ")
                .build()
                .edit()
                .as_fixed_width(5)
                .skip_cells(3)
                .fg(2)
                .bg(7)
                .add_text("BCDEF") // Try to add more text than should fit
                .build();
            println!("Case 4: Convert to larger width and add text (should truncate)");
            assert_eq!(expected, actual);
        }

        #[test]
        fn as_dynamic() {
            // Test converting from fixed size to dynamic
            #[rustfmt::skip]
            let initial = charmi_line(&[
                'A' as u32, 1, 6, 0,
                'B' as u32, 1, 6, 0,
                'C' as u32, 3, 8, 0,
            ]);

            #[rustfmt::skip]
            let expected = charmi_line(&[
                'A' as u32, 1, 6, 0,
                'B' as u32, 1, 6, 0,
                'C' as u32, 3, 8, 0,
                'D' as u32, 2, 7, 0,
                'E' as u32, 2, 7, 0,
            ]);

            let actual = CharmiBuilder::edit(initial)
                .as_dynamic_size()
                .skip_cells(3)
                .fg(2)
                .bg(7)
                .add_text("DE")
                .build();

            println!("Case 1");
            assert_eq!(expected, actual);

            // Test converting from fixed width to dynamic
            let actual = CharmiBuilder::fixed_width(3)
                .fg(1)
                .bg(6)
                .add_text("AB")
                .fg(3)
                .bg(8)
                .add_text("C")
                .as_dynamic_size()
                .fg(2)
                .bg(7)
                .add_text("DE")
                .build();

            println!("Case 2");
            assert_eq!(expected, actual);

            // Test dynamic to dynamic (should maintain content but update fill)
            let actual = CharmiBuilder::dynamic()
                .fg(1)
                .bg(6)
                .add_text("AB")
                .with_fill('-')
                .as_dynamic_size()
                .with_fill(CharCell::new_chfgbg('C', 3, 8))
                .gap_char('#')
                .fg(2)
                .bg(7)
                .add_text("#DE")
                .build();

            println!("Case 3");
            assert_eq!(expected, actual);
        }
    }

    mod fill {
        use super::*;

        #[test]
        fn fill_dynamic() {
            #[rustfmt::skip]
            let expected = charmi_line(&[
                'A' as u32,  2,  7, 0,
                'B' as u32,  2,  7, 0,
                0        , NC, NC, 0,
                0        , NC, NC, 0,
            ]);

            let actual = CharmiBuilder::dynamic()
                .fg(2)
                .bg(7)
                .add_text("AB")
                .skip_cells(2) // Force fill to appear
                .build();
            println!("Case 1: Default fill is GAP");
            assert_eq!(expected, actual);

            #[rustfmt::skip]
            let expected = charmi_line(&[
                'A' as u32,  2,  7, 0,
                'B' as u32,  2,  7, 0,
                '-' as u32,  2,  7, 0,
                '-' as u32,  2,  7, 0,
            ]);

            let actual = CharmiBuilder::dynamic()
                .fg(2)
                .bg(7)
                .with_fill('-')
                .add_text("AB")
                .skip_cells(2)
                .build();
            println!("Case 2: Fill inherits builder style");
            assert_eq!(expected, actual);

            #[rustfmt::skip]
            let expected = charmi_line(&[
                'A' as u32,  2,  7, 0,
                'B' as u32,  2,  7, 0,
                '-' as u32,  1,  6, 0,
                '-' as u32,  1,  6, 0,
            ]);

            let fill_cell = CharCell::new_chfgbg('-', 1, 6);
            let actual = CharmiBuilder::dynamic()
                .fg(2)
                .bg(7)
                .with_fill(fill_cell)
                .add_text("AB")
                .skip_cells(2)
                .build();
            println!("Case 3: CharCell fill maintains its style");
            assert_eq!(expected, actual);

            #[rustfmt::skip]
            let expected = charmi_line(&[
                'A' as u32,  2,  7, 0,
                'B' as u32,  2,  7, 0,
                '-' as u32,  1,  6, 0,
                '-' as u32,  1,  6, 0,
                '+' as u32,  3,  8, 0,
                '+' as u32,  3,  8, 0,
            ]);

            let actual = CharmiBuilder::dynamic()
                .fg(1)
                .bg(6)
                .with_fill('-')
                .fg(2)
                .bg(7)
                .add_text("AB")
                .skip_cells(2)
                .build()
                .edit()
                .as_dynamic_size()
                .fg(3)
                .bg(8)
                .with_fill('+')
                .skip_cells(6)
                .build();
            println!("Case 4: Fill applies to expanded area after edit");
            assert_eq!(expected, actual);
        }

        #[test]
        fn fill_fixed_width() {
            #[rustfmt::skip]
            let expected = charmi_line(&[
                'A' as u32,  1,  6, 0,
                'B' as u32,  1,  6, 0,
                0        , NC, NC, 0,
                0        , NC, NC, 0,
            ]);

            let actual = CharmiBuilder::fixed_width(4)
                .fg(1)
                .bg(6)
                .add_text("AB")
                .build();
            println!("Case 1: Default fill is GAP");
            assert_eq!(expected, actual);

            #[rustfmt::skip]
            let expected = charmi_line(&[
                'A' as u32,  2,  7, 0,
                'B' as u32,  2,  7, 0,
                '-' as u32,  2,  7, 0,
                '-' as u32,  2,  7, 0,
            ]);

            let actual = CharmiBuilder::fixed_width(4)
                .fg(2)
                .bg(7)
                .with_fill('-')
                .add_text("AB")
                .build();
            println!("Case 2: Fill inherits builder style");
            assert_eq!(expected, actual);

            #[rustfmt::skip]
            let expected = charmi_line(&[
                'A' as u32,  2,  7, 0,
                'B' as u32,  2,  7, 0,
                '-' as u32,  1,  6, 0,
                '-' as u32,  1,  6, 0,
            ]);

            let fill_cell = CharCell::new_chfgbg('-', 1, 6);
            let actual = CharmiBuilder::fixed_width(4)
                .fg(2)
                .bg(7)
                .with_fill(fill_cell)
                .add_text("AB")
                .build();
            println!("Case 3: CharCell fill ignores builder style");
            assert_eq!(expected, actual);

            #[rustfmt::skip]
            let expected = charmi_line(&[
                'A' as u32,  2,  7, 0,
                'B' as u32,  2,  7, 0,
                '-' as u32,  1,  6, 0,
                '-' as u32,  1,  6, 0,
                '+' as u32,  3,  8, 0,
                '+' as u32,  3,  8, 0,
            ]);

            let actual = CharmiBuilder::fixed_width(4)
                .fg(1)
                .bg(6)
                .with_fill('-')
                .fg(2)
                .bg(7)
                .add_text("AB")
                .build()
                .edit()
                .as_fixed_width(6)
                .fg(3)
                .bg(8)
                .with_fill('+')
                .build();
            println!("Case 4: Fill applies to increased width after edit");
            assert_eq!(expected, actual);
        }

        #[test]
        fn fill_fixed_size() {
            // Test default fill (GAP)
            #[rustfmt::skip]
            let expected = charmi_double_line(&[
                'A' as u32,  1,  6, 0,
                'B' as u32,  1,  6, 0,
                0        , NC, NC, 0,
                0        , NC, NC, 0,
                0        , NC, NC, 0,
                0        , NC, NC, 0,
                0        , NC, NC, 0,
                0        , NC, NC, 0,
            ]);

            let actual = CharmiBuilder::fixed_size(4, 2)
                .fg(1)
                .bg(6)
                .add_text("AB")
                .build();
            println!("Case 1: Default fill is GAP");
            assert_eq!(expected, actual);

            // Test char fill inherits builder style
            #[rustfmt::skip]
            let expected = charmi_double_line(&[
                'A' as u32,  2,  7, 0,
                'B' as u32,  2,  7, 0,
                '-' as u32,  2,  7, 0,  // Fill inherits style
                '-' as u32,  2,  7, 0,
                '-' as u32,  2,  7, 0,
                '-' as u32,  2,  7, 0,
                '-' as u32,  2,  7, 0,
                '-' as u32,  2,  7, 0,
            ]);

            let actual = CharmiBuilder::fixed_size(4, 2)
                .fg(2)
                .bg(7)
                .with_fill('-')
                .add_text("AB")
                .build();
            println!("Case 2: Fill inherits builder style");
            assert_eq!(expected, actual);

            // Test CharCell fill keeps its own style
            #[rustfmt::skip]
            let expected = charmi_double_line(&[
                'A' as u32,  2,  7, 0,
                'B' as u32,  2,  7, 0,
                '-' as u32,  1,  6, 0,  // Fill keeps original style
                '-' as u32,  1,  6, 0,
                '-' as u32,  1,  6, 0,
                '-' as u32,  1,  6, 0,
                '-' as u32,  1,  6, 0,
                '-' as u32,  1,  6, 0,
            ]);

            let fill_cell = CharCell::new_chfgbg('-', 1, 6);
            let actual = CharmiBuilder::fixed_size(4, 2)
                .fg(2)
                .bg(7)
                .with_fill(fill_cell)
                .add_text("AB")
                .build();
            println!("Case 3: CharCell fill ignores builder style");
            assert_eq!(expected, actual);

            // Test fill applies to expanded size
            #[rustfmt::skip]
            let expected = charmi_double_line(&[
                'A' as u32,  2,  7, 0,
                'B' as u32,  2,  7, 0,
                '+' as u32,  3,  8, 0,  // New fill after size increase
                '+' as u32,  3,  8, 0,
                '-' as u32,  1,  6, 0,  // Original fill
                '-' as u32,  1,  6, 0,
                '+' as u32,  3,  8, 0,
                '+' as u32,  3,  8, 0,
            ]);

            let actual = CharmiBuilder::fixed_size(2, 2)
                .fg(1)
                .bg(6)
                .with_fill('-')
                .fg(2)
                .bg(7)
                .add_text("AB")
                .build()
                .edit()
                .as_fixed_size(4, 2)
                .fg(3)
                .bg(8)
                .with_fill('+')
                .build();
            println!("Case 4: Fill applies to expanded size after edit");
            assert_eq!(expected, actual);
        }
    }

    mod clipping {
        use super::*;

        #[test]
        fn clipping_does_not_change_fill_behavior() {
            println!("Case 1: Clipping doesn't apply fill before build");
            #[rustfmt::skip]
            let expected = charmi_line(&[
                'A' as u32, 15, 0, 0,
                'C' as u32, 8, 0, 0,
                'D' as u32, 15, 0, 0,
                'C' as u32, 8, 0, 0,
            ]);
            let actual = CharmiBuilder::dynamic()
                .gap_char('#')
                .add_text("A#D")
                .clip(0, 0, 4, 1)
                .fg(8)
                .with_fill('C')
                .build();
            assert_eq!(expected, actual);

            println!("Case 2: clip_intersect doesn't apply fill before build");
            let actual = CharmiBuilder::dynamic()
                .gap_char('#')
                .add_text("A#D#D#")
                .clip_intersect(0, 0, 4, 1)
                .fg(8)
                .with_fill('C')
                .build();
            assert_eq!(expected, actual);
        }

        #[test]
        fn clipping_within_bounds() {
            println!("Case 1: clip within bounds");
            #[rustfmt::skip]
            let expected = charmi_line(&[
                'B' as u32, 15, 0, 0,
                'C' as u32, 15, 0, 0,
            ]);
            let actual = CharmiBuilder::dynamic()
                .add_text("ABCD")
                .clip(1, 0, 2, 1)
                .build();
            assert_eq!(expected, actual);

            println!("Case 2: clip_intersect within bounds");
            let actual = CharmiBuilder::dynamic()
                .add_text("ABCD")
                .clip_intersect(1, 0, 2, 1)
                .build();
            assert_eq!(expected, actual);
        }

        #[test]
        fn clipping_partial_intersection() {
            println!("Case 1: clip with x negative, but width extends into the image");
            #[rustfmt::skip]
            let expected = charmi_line(&[
                0        , NC, NC, 0,
                'A' as u32, 15, 0, 0,
                'B' as u32, 15, 0, 0,
            ]);
            let actual = CharmiBuilder::dynamic()
                .add_text("ABCD")
                .clip(-1, 0, 3, 1)
                .build();
            assert_eq!(expected, actual);

            println!("Case 2: clip with y negative, but height extends into image");
            #[rustfmt::skip]
            let expected = charmi_double_line(&[
                0         , NC, NC, 0,
                0         , NC, NC, 0,
                'A' as u32, 15, 0, 0,
                'B' as u32, 15, 0, 0,
            ]);
            let actual = CharmiBuilder::dynamic()
                .add_text("AB")
                .clip(0, -1, 2, 2)
                .build();
            assert_eq!(expected, actual);

            println!("Case 3: clip where x + width exceeds original size");
            #[rustfmt::skip]
            let expected = charmi_line(&[
                'C' as u32, 15, 0, 0,
                0        , NC, NC, 0,
            ]);
            let actual = CharmiBuilder::dynamic()
                .add_text("ABC")
                .clip(2, 0, 2, 1)
                .build();
            assert_eq!(expected, actual);

            println!("Case 4: clip_intersect where x + width exceeds original size");
            #[rustfmt::skip]
            let expected = charmi_line(&[
                'C' as u32, 15, 0, 0,
            ]);
            let actual = CharmiBuilder::dynamic()
                .add_text("ABC")
                .clip_intersect(2, 0, 2, 1)
                .build();
            assert_eq!(expected, actual);

            println!("Case 5: clip where y + height exceeds original size");
            #[rustfmt::skip]
            let expected = charmi_double_line(&[
                'A' as u32, 15, 0, 0,
                'B' as u32, 15, 0, 0,
                0        , NC, NC, 0,
                0        , NC, NC, 0,
            ]);
            let actual = CharmiBuilder::dynamic()
                .add_text("AB")
                .clip(0, 0, 2, 2)
                .build();
            assert_eq!(expected, actual);

            println!("Case 6: clip_intersect where y + height exceeds original size");
            #[rustfmt::skip]
            let expected = charmi_line(&[
                'A' as u32, 15, 0, 0,
                'B' as u32, 15, 0, 0,
            ]);
            let actual = CharmiBuilder::dynamic()
                .add_text("AB")
                .clip_intersect(0, 0, 2, 2)
                .build();
            assert_eq!(expected, actual);
        }

        #[test]
        fn clipping_double_width_characters() {
            println!("Case 1: clip that starts before and ends inside a double-width char");
            #[rustfmt::skip]
            let expected = charmi_double_line(&[
                'A' as u32,  15, 0, 0,
                '<' as u32,  15, 0, 0,
                'バ' as u32, 15, 0, 0,
                SC         , 62, 60, 0,
            ]);
            let actual = CharmiBuilder::dynamic()
                .split_chars('<', '>')
                .add_line("Aバ")
                .add_text("バ")
                .clip(0, 0, 2, 2)
                .build();
            assert_eq!(expected, actual);

            println!(
                "Case 2: clip_intersect that starts before and ends inside a double-width char"
            );
            let actual = CharmiBuilder::dynamic()
                .split_chars('<', '>')
                .add_line("Aバ")
                .add_text("バ")
                .clip_intersect(0, 0, 2, 2)
                .build();
            assert_eq!(expected, actual);

            println!("Case 3: clip that starts inside and ends after a double-width char");
            #[rustfmt::skip]
            let expected = charmi_double_line(&[
                '>' as u32, 15, 0, 0,
                'C' as u32, 15, 0, 0,
                'バ' as u32, 15, 0, 0,
                SC         , 62, 60, 0,
            ]);
            let actual = CharmiBuilder::dynamic()
                .split_chars('<', '>')
                .add_line("バC")
                .add_text(" バ")
                .clip(1, 0, 2, 2)
                .build();
            assert_eq!(expected, actual);

            println!(
                "Case 4: clip_intersect that starts inside and ends after a double-width char"
            );
            let actual = CharmiBuilder::dynamic()
                .split_chars('<', '>')
                .add_line("バC")
                .add_text(" バ")
                .clip_intersect(1, 0, 2, 2)
                .build();
            assert_eq!(expected, actual);

            println!("Case 5: clip that contains only the trailing half of a double-width char");
            #[rustfmt::skip]
            let expected = charmi_line(&[
                '>' as u32, 15, 0, 0,
            ]);
            let actual = CharmiBuilder::dynamic()
                .split_chars('<', '>')
                .add_text("AバC")
                .clip(2, 0, 1, 1)
                .build();
            assert_eq!(expected, actual);

            println!("Case 6: clip_intersect that contains only the trailing half of a double-width char");
            let actual = CharmiBuilder::dynamic()
                .split_chars('<', '>')
                .add_text("AバC")
                .clip_intersect(2, 0, 1, 1)
                .build();
            assert_eq!(expected, actual);

            println!("Case 7: clip a double-width character exactly");
            #[rustfmt::skip]
            let expected = charmi_line(&[
                'バ' as u32, 15,  0, 0,
                SC         , 62, 60, 0,
            ]);
            let actual = CharmiBuilder::dynamic()
                .split_chars('<', '>')
                .add_text("バ")
                .clip(0, 0, 2, 1)
                .build();
            assert_eq!(expected, actual);

            println!("Case 8: clip_intersect a double-width character exactly");
            let actual = CharmiBuilder::dynamic()
                .split_chars('<', '>')
                .add_text("バ")
                .clip_intersect(0, 0, 2, 1)
                .build();
            assert_eq!(expected, actual);
        }

        #[test]
        fn clipping_zero_dimensions() {
            println!("Case 1: clip with zero width");
            let expected = CharmiImage::new_empty(0, 0);
            let actual = CharmiBuilder::fixed_size(2, 2).clip(0, 0, 0, 2).build();
            assert_eq!(expected, actual);

            println!("Case 2: clip with zero height");
            let actual = CharmiBuilder::fixed_size(2, 2).clip(0, 0, 2, 0).build();
            assert_eq!(expected, actual);

            println!("Case 3: clip_intersect with zero width");
            let actual = CharmiBuilder::fixed_size(2, 2)
                .clip_intersect(0, 0, 0, 2)
                .build();
            assert_eq!(expected, actual);

            println!("Case 4: clip_intersect with zero height");
            let actual = CharmiBuilder::fixed_size(2, 2)
                .clip_intersect(0, 0, 2, 0)
                .build();
            assert_eq!(expected, actual);
        }

        #[test]
        fn clipping_completely_outside() {
            println!("Case 1: clip with x and y negative, entirely out of bounds");

            #[rustfmt::skip]
            let expected = charmi_double_line(&[
                0, NC, NC, 0,  0, NC, NC, 0,
                0, NC, NC, 0,  0, NC, NC, 0,
                0, NC, NC, 0,  0, NC, NC, 0,
                0, NC, NC, 0,  0, NC, NC, 0,
            ]);
            let actual = CharmiBuilder::fixed_size(2, 2)
                .add_text("AB")
                .clip(-5, -5, 4, 2)
                .build();
            assert_eq!(expected, actual);

            println!("Case 2: clip with x and y positive, entirely out of bounds");
            let actual = CharmiBuilder::fixed_size(2, 2)
                .add_text("AB")
                .clip(5, 5, 4, 2)
                .build();
            assert_eq!(expected, actual);

            println!("Case 3: clip_intersect with x and y negative, entirely out of bounds");
            let expected = CharmiImage::new_empty(0, 0);
            let actual = CharmiBuilder::fixed_size(2, 2)
                .add_text("AB")
                .clip_intersect(5, 5, 4, 2)
                .build();
            assert_eq!(expected, actual);

            println!("Case 4: clip_intersect with x and y positive, entirely out of bounds");
            let actual = CharmiBuilder::fixed_size(2, 2)
                .add_text("AB")
                .clip_intersect(5, 5, 4, 2)
                .build();
            assert_eq!(expected, actual);

            println!("Case 5: clip_intersect on an empty builder");
            let actual = CharmiBuilder::fixed_size(0, 0)
                .clip_intersect(0, 0, 1, 1)
                .build();
            assert_eq!(expected, actual);

            println!("Case 6: clip on an empty builder");
            let actual = CharmiBuilder::fixed_size(0, 0).clip(0, 0, 1, 1).build();
            let expected = charmi_line(&[0, NC, NC, 0]);
            assert_eq!(expected, actual);
        }
    }

    mod draw {
        use super::*;

        #[test]
        fn draw_in_bounds() {
            #[rustfmt::skip]
            let image_to_draw = charmi_double_line(&[
                'B' as u32, 1, 2, 3,
                'C' as u32, 4, 5, 6,
                'X' as u32, 7, 8, 9,
                'Y' as u32, 0, 0, 0,
            ]);

            #[rustfmt::skip]
            let expected = unsafe {
                CharmiImage::from_raw_u32(5, 2, &[
                    'A' as u32, 15,  0, 0,
                    'B' as u32, 1, 2, 3,
                    'C' as u32, 4, 5, 6,
                    0,          NC, NC, 0,
                    0,          NC, NC, 0,

                    0,          NC, NC, 0,
                    'X' as u32, 7, 8, 9,
                    'Y' as u32, 0, 0, 0,
                    'Z' as u32, 15, 0, 0,
                    0,          NC, NC, 0,
                ])
            };

            println!("Case 1 - Fixed size");
            let actual_fixed_size = CharmiBuilder::fixed_size(5, 2)
                .add_text("A")
                .draw(&image_to_draw)
                .add_text("Z")
                .build();
            assert_eq!(expected, actual_fixed_size);

            println!("Case 2 - Fixed width");
            let actual_fixed_width = CharmiBuilder::fixed_width(5)
                .add_text("A")
                .draw(&image_to_draw)
                .add_text("Z")
                .build();
            assert_eq!(expected, actual_fixed_width);

            println!("Case 3 - Dynamic");
            let actual_dynamic = CharmiBuilder::dynamic()
                .add_text("A")
                .draw(&image_to_draw)
                .add_text("Z")
                .add_gap(1) // To pad to 5 width
                .build();
            assert_eq!(expected, actual_dynamic);
        }

        #[test]
        fn draw_off_edges() {
            #[rustfmt::skip]
            let image_to_draw = charmi_double_line(&[
                'B' as u32, 1, 2, 3,
                'C' as u32, 4, 5, 6,
                'D' as u32, 7, 8, 9,
                'X' as u32, 7, 8, 9,
                'Y' as u32, 0, 0, 0,
                'Z' as u32, 4, 3, 2,
            ]);

            #[rustfmt::skip]
            let expected = unsafe {
                CharmiImage::from_raw_u32(3, 2, &[
                    'A' as u32, 15,  0, 0,
                    'B' as u32, 1, 2, 3,
                    'C' as u32, 4, 5, 6,

                    0,          NC, NC, 0,
                    'X' as u32, 7, 8, 9,
                    'Y' as u32, 0, 0, 0,
                ])
            };

            println!("Case 1 - Fixed size, off right edge. Cursor not at 0,0");
            let actual = CharmiBuilder::fixed_size(3, 2)
                .add_text("A")
                .draw(&image_to_draw)
                .build();
            assert_eq!(expected, actual);

            println!("Case 2 - Fixed width, off right edge. Cursor not at 0,0");
            let actual = CharmiBuilder::fixed_width(3)
                .add_text("A")
                .draw(&image_to_draw)
                .build();
            assert_eq!(actual, expected);

            #[rustfmt::skip]
            let expected = unsafe {
                CharmiImage::from_raw_u32(3, 2, &[
                    0,          NC, NC, 0,
                    0,          NC, NC, 0,
                    0,          NC, NC, 0,
                    'A' as u32, 15,  0, 0,
                    'B' as u32, 1, 2, 3,
                    'C' as u32, 4, 5, 6,
                ])
            };

            println!("Case 3 - Fixed size, off right and bottom edge. Cursor not at 0,0");
            let actual = CharmiBuilder::fixed_size(3, 2)
                .next_line()
                .add_text("A")
                .draw(&image_to_draw)
                .build();
            assert_eq!(expected, actual);
        }

        #[test]
        fn draw_multiple() {
            #[rustfmt::skip]
            let image_to_draw = charmi_double_line(&[
                'B' as u32, 1, 2, 3,
                'C' as u32, 4, 5, 6,
                'D' as u32, 7, 8, 9,
                'E' as u32, 0, 0, 0,
            ]);

            #[rustfmt::skip]
            let image_to_draw_second = charmi_double_line(&[
                'X' as u32, 1, 2, 3,
                'Y' as u32, 4, 5, 6,
                'Z' as u32, 7, 8, 9,
                'Q' as u32, 0, 0, 0,
            ]);

            println!("Case 1 - Draw multiple images sequentially");
            #[rustfmt::skip]
            let expected = unsafe {
                CharmiImage::from_raw_u32(4, 3, &[
                    'B' as u32, 1, 2, 3,
                    'C' as u32, 4, 5, 6,
                    0,          NC, NC, 0,
                    0,          NC, NC, 0,

                    'D' as u32, 7, 8, 9,
                    'E' as u32, 0, 0, 0,
                    'X' as u32, 1, 2, 3,
                    'Y' as u32, 4, 5, 6,

                    0,          NC, NC, 0,
                    0,          NC, NC, 0,
                    'Z' as u32, 7, 8, 9,
                    'Q' as u32, 0, 0, 0,
                ])
            };
            let actual = CharmiBuilder::dynamic()
                .draw(&image_to_draw)
                .draw(&image_to_draw_second)
                .build();
            assert_eq!(expected, actual);

            #[rustfmt::skip]
            let expected = unsafe {
                CharmiImage::from_raw_u32(2, 4, &[
                    'B' as u32, 1, 2, 3,
                    'C' as u32, 4, 5, 6,

                    'D' as u32, 7, 8, 9,
                    'E' as u32, 0, 0, 0,

                    'X' as u32, 1, 2, 3,
                    'Y' as u32, 4, 5, 6,

                    'Z' as u32, 7, 8, 9,
                    'Q' as u32, 0, 0, 0,
                ])
            };
            println!("Case 2 - Draw after next_line()");
            let actual = CharmiBuilder::dynamic()
                .draw(&image_to_draw)
                .next_line()
                .draw(&image_to_draw_second)
                .build();
            assert_eq!(expected, actual);
        }

        #[test]
        fn draw_empty() {
            let empty = CharmiImage::new_empty(0, 0);

            println!("Case 1 - Drawing an empty image does nothing");
            let expected = CharmiBuilder::fixed_size(3, 2).add_text("ABC").build();
            let actual = CharmiBuilder::fixed_size(3, 2)
                .add_text("AB")
                .draw(&empty)
                .add_text("C")
                .build();
            assert_eq!(expected, actual);

            println!("Case 2 - Drawing onto an empty builder does nothing");
            let actual = CharmiBuilder::fixed_size(0, 0).draw(&empty).build();
            assert_eq!(empty, actual);
        }

        #[test]
        fn draw_empty_respects_underlying_characters() {
            #[rustfmt::skip]
            let expected = charmi_line(&[
                'A' as u32, 15, 0, 0,
                'B' as u32, 15, 0, 0,
                'C' as u32, 15, 0, 0,
            ]);
            let empty = CharmiImage::new_empty(2, 1);

            println!("Case 1 - Drawing an empty image respects underlying image");
            let actual = CharmiBuilder::fixed_size(3, 1)
                .add_text("ABC")
                .set_cursor(1, 0)
                .draw(&empty)
                .build();
            assert_eq!(expected, actual);

            #[rustfmt::skip]
            let expected = charmi_line(&[
                'バ' as u32, 15, 0, 0,
                SC,          32, 32, 0,
                'カ' as u32, 15, 0, 0,
                SC,          32, 32, 0,
            ]);
            let empty = CharmiImage::new_empty(3, 1);

            println!("Case 2 - Drawing an empty image respects underlying image - unicode");
            let actual = CharmiBuilder::fixed_size(4, 1)
                .add_text("バカ")
                .set_cursor(1, 0)
                .draw(&empty)
                .build();
            assert_eq!(expected, actual);
        }

        #[test]
        fn draw_double_width() {
            #[rustfmt::skip]
            let image_to_draw = charmi_double_line(&[
                'バ' as u32, 1, 2, 3,
                SC         , 62, 60, 6,
                'カ' as u32, 1, 2, 3,
                SC         , 62, 60, 6,
                'C' as u32, 7, 8, 9,
                'め' as u32, 1, 2, 3,
                SC         , 62, 60, 6,
                'D' as u32, 0, 0, 0,
            ]);

            println!("Case 1 - Source image with double-width characters");
            #[rustfmt::skip]
            let expected = charmi_double_line(&[
                'A' as u32, 15,  0, 0,
                'バ' as u32, 1, 2, 3,
                SC         , 62, 60, 6,
                'カ' as u32, 1, 2, 3,
                SC         , 62, 60, 6,
                0         , NC, NC, 0,
                'C' as u32, 7, 8, 9,
                'め' as u32, 1, 2, 3,
                SC         , 62, 60, 6,
                'D' as u32, 0, 0, 0,
            ]);

            let actual = CharmiBuilder::fixed_size(5, 2)
                .add_text("A")
                .draw(&image_to_draw)
                .add_text("?")
                .build();
            assert_eq!(expected, actual);

            println!("Case 2 - Truncated source image");
            #[rustfmt::skip]
            let expected = charmi_double_line( &[
                'A' as u32, 15,  0, 0,
                'バ' as u32, 1, 2, 3,
                SC         , 62, 60, 6,
                '<' as u32, 1, 2, 3,
                0         , NC, NC, 0,
                'C' as u32, 7, 8, 9,
                'め' as u32, 1, 2, 3,
                SC         , 62, 60, 6,
            ]);

            let actual = CharmiBuilder::fixed_size(4, 2)
                .add_text("A")
                .draw(&image_to_draw)
                .add_text("?")
                .build();
            assert_eq!(expected, actual);

            println!("Case 3- Cracking double width characters on existance image");
            #[rustfmt::skip]
            let expected = charmi_line(&[
                '|' as u32, 15,  0, 0,
                '>' as u32, 15,  0, 0,
                '<' as u32, 15,  0, 0,
                '|' as u32, 15,  0, 0,
            ]);
            let image_to_draw = CharmiBuilder::dynamic()
                .add_text("|")
                .add_gap(2)
                .add_text("|")
                .build();
            let actual = CharmiBuilder::dynamic()
                .split_chars('<', '>')
                .add_text("バカ")
                .set_cursor(0, 0)
                .draw(&image_to_draw)
                .build();
            assert_eq!(expected, actual);
        }
    }

    mod add_text {
        use super::*;

        #[test]
        fn fixed_size_happy_path() {
            #[rustfmt::skip]
            let expected = charmi_line(&[
                '<' as u32, 1, 6, 0,
                '-' as u32, 1, 6, 0,
                '-' as u32, 1, 6, 0,
                '-' as u32, 1, 6, 0,
                '>' as u32, 1, 6, 0,
            ]);

            let actual = CharmiBuilder::fixed_size(5, 1)
                .fg(1)
                .bg(6)
                .add_text("<--->")
                .build();
            assert_eq!(expected, actual);
        }

        #[test]
        fn fixed_width_happy_path() {
            #[rustfmt::skip]
            let expected = charmi_line(&[
                '<' as u32, 1, 6, 0,
                '-' as u32, 1, 6, 0,
                '-' as u32, 1, 6, 0,
                '-' as u32, 1, 6, 0,
                '>' as u32, 1, 6, 0,
            ]);

            let actual = CharmiBuilder::fixed_width(5)
                .fg(1)
                .bg(6)
                .add_text("<--->")
                .build();
            assert_eq!(expected, actual);
        }

        #[test]
        fn dynamic_happy_path() {
            #[rustfmt::skip]
            let expected = charmi_line(&[
                '<' as u32, 1, 6, 0,
                '-' as u32, 1, 6, 0,
                '-' as u32, 1, 6, 0,
                '-' as u32, 1, 6, 0,
                '>' as u32, 1, 6, 0,
            ]);

            let actual = CharmiBuilder::dynamic()
                .fg(1)
                .bg(6)
                .add_text("<--->")
                .build();
            assert_eq!(expected, actual);
        }

        #[test]
        fn truncated() {
            #[rustfmt::skip]
            let expected = charmi_line(&[
                '<' as u32, 1, 6, 0,
                '-' as u32, 1, 6, 0,
                '-' as u32, 1, 6, 0
            ]);

            let actual = CharmiBuilder::fixed_size(3, 1)
                .fg(1)
                .bg(6)
                .add_text("<--->")
                .build();
            assert_eq!(expected, actual);

            let actual = CharmiBuilder::fixed_width(3)
                .fg(1)
                .bg(6)
                .add_text("<--->")
                .build();
            assert_eq!(expected, actual);
        }

        #[test]
        fn unicode_sanitized() {
            #[rustfmt::skip]
            let expected = charmi_line(&[
                'バ' as u32 ,  1,  6, 0,
                SC          , 32, 32, 0,
                'カ' as u32 ,  1,  6, 0,
                SC          , 32, 32, 0,
            ]);

            let actual = CharmiBuilder::fixed_size(4, 1)
                .fg(1)
                .bg(6)
                .add_text("バカ")
                .build();
            assert_eq!(expected, actual);
        }

        #[test]
        fn unicode_overwrite_cracking() {
            #[rustfmt::skip]
            let expected = charmi_line(&[
                '#' as u32 ,  1,  6, 0,
                'バ' as u32 ,  2,  6, 0,
                SC          , 32, 32, 0,
                '#' as u32 ,  1,  6, 0,
            ]);

            let actual = CharmiBuilder::fixed_size(4, 1)
                .fg(1)
                .bg(6)
                .split_char('#')
                .add_text("バカ")
                .build()
                .edit()
                .skip_cells(1)
                .fg(2)
                .bg(6)
                .add_text("バ")
                .build();
            assert_eq!(expected, actual);
        }

        #[test]
        fn unicode_overwrite_no_cracking() {
            #[rustfmt::skip]
            let expected = charmi_line(&[
                'バ' as u32 ,  2,  6, 0,
                SC          , 32, 32, 0,
                'カ' as u32 ,  1,  6, 0,
                SC          , 32, 32, 0,
            ]);

            let actual = CharmiBuilder::fixed_size(4, 1)
                .fg(1)
                .bg(6)
                .add_text("バカ")
                .build()
                .edit()
                .fg(2)
                .bg(6)
                .add_text("バ")
                .build();
            assert_eq!(expected, actual);
        }

        #[test]
        fn unicode_truncated() {
            #[rustfmt::skip]
            let expected = charmi_line(&[
                'バ' as u32,  1        ,  6        , 0,
                SC         , '#' as u32, '#' as u32, 0,
                '#' as u32 ,  1        ,  6        , 0,
            ]);

            let actual = CharmiBuilder::fixed_size(3, 1)
                .fg(1)
                .bg(6)
                .split_char('#')
                .add_text("バカ")
                .build();

            assert_eq!(expected, actual);

            let actual = CharmiBuilder::fixed_width(3)
                .fg(1)
                .bg(6)
                .split_char('#')
                .add_text("バカ")
                .build();
            assert_eq!(expected, actual);

            let actual = CharmiBuilder::fixed_width(3)
                .add_text("???")
                .build()
                .edit()
                .fg(1)
                .bg(6)
                .split_char('#')
                .add_text("バカ")
                .build();
            assert_eq!(expected, actual);
        }

        #[test]
        fn when_row_is_full() {
            #[rustfmt::skip]
            let expected = charmi_line(&[
                '?' as u32,  15        ,  0        , 0,
                '?' as u32,  15        ,  0        , 0,
                '!' as u32,  15        ,  0        , 0,
            ]);

            let actual = CharmiBuilder::fixed_width(3)
                .add_text("??!")
                .add_text("HELLO")
                .build();
            assert_eq!(expected, actual);

            let actual = CharmiBuilder::fixed_size(3, 1)
                .add_text("??!")
                .add_text("HELLO")
                .build();
            assert_eq!(expected, actual);
        }

        #[test]
        fn when_text_is_empty() {
            #[rustfmt::skip]
            let expected = charmi_line(&[
                0, NC, NC, 0,
                0, NC, NC, 0,
                0, NC, NC, 0,
            ]);

            let actual = CharmiBuilder::fixed_width(3).add_text("").build();
            assert_eq!(expected, actual);

            let actual = CharmiBuilder::fixed_size(3, 1).add_text("").build();
            assert_eq!(expected, actual);
        }

        #[test]
        fn proper_repositioning() {
            #[rustfmt::skip]
            let expected = charmi_line(&[
                '<' as u32, 15, 0, 0,
                '-' as u32, 15, 0, 0,
                '-' as u32, 15, 0, 0,
                '-' as u32, 15, 0, 0,
                '>' as u32, 15, 0, 0,
            ]);

            let actual = CharmiBuilder::fixed_width(5)
                .add_text("<-")
                .add_text("-->")
                .build();
            assert_eq!(expected, actual);

            let actual = CharmiBuilder::fixed_size(5, 1)
                .add_text("<-")
                .add_text("-->")
                .build();
            assert_eq!(expected, actual);

            let actual = CharmiBuilder::fixed_width(5)
                .add_text("NOTME")
                .build()
                .edit()
                .as_dynamic_size()
                .add_text("<")
                .add_text("-")
                .add_text("--")
                .add_text(">")
                .build();
            assert_eq!(expected, actual);

            let actual = CharmiBuilder::dynamic()
                .add_text("NOT ME!")
                .build()
                .edit()
                .as_fixed_width(5)
                .add_text("<-")
                .add_text("-->")
                .build();
            assert_eq!(expected, actual);
        }

        #[test]
        fn with_empty_chars() {
            #[rustfmt::skip]
            let expected = charmi_line(&[
                '死' as u32,        15,          0, 0,
                SC        , '>' as u32, '<' as u32, 0,
                '<' as u32,         15,          0, 0,
                '3' as u32,         15,          0, 0,
                '3' as u32,         15,          0, 0,
                '>' as u32,         15,          0, 0,
                '?' as u32,         15,          0, 0,
            ]);

            let actual = CharmiBuilder::dynamic()
                .split_chars('<', '>')
                .add_text("死ねが!") // お前が
                .build()
                .edit()
                .gap_char('#')
                .add_text("#")
                .add_text("##33#?")
                .build();
            assert_eq!(expected, actual);
        }

        #[test]
        fn off_the_edge() {
            let expected = charmi_line(&[
                '<' as u32, 15, 0, 0, '-' as u32, 15, 0, 0, '>' as u32, 15, 0, 0,
            ]);

            let actual = CharmiBuilder::fixed_width(3)
                .add_text("<->DEMONS") // お前が
                .build();
            assert_eq!(expected, actual);
            let actual = CharmiBuilder::fixed_size(3, 1)
                .add_text("<->DEMONS")
                .build();
            assert_eq!(expected, actual);

            let actual = CharmiBuilder::fixed_width(3)
                .add_text("<->")
                .add_text("DEMONS")
                .build();
            assert_eq!(expected, actual);
            let actual = CharmiBuilder::fixed_size(3, 1)
                .add_text("<->")
                .add_text("DEMONS")
                .build();
            assert_eq!(expected, actual);

            let actual = CharmiBuilder::fixed_size(3, 1)
                .add_text("<->")
                .next_line()
                .add_text("DEMONS")
                .build();
            assert_eq!(expected, actual);
        }
    }

    mod add_text_alternatives {
        use super::*;

        #[test]
        fn add_gap_happy_path() {
            #[rustfmt::skip]
            let expected = charmi_line(&[
                '|' as u32, 15,  0, 0,
                0         , NC, NC, 0,
                0         , NC, NC, 0,
                '|' as u32, 15,  0, 0,
            ]);

            let actual = CharmiBuilder::dynamic()
                .add_text("|")
                .add_gap(2)
                .add_text("|")
                .build();
            assert_eq!(expected, actual);
        }

        #[test]
        fn add_gap_overwrites_previous() {
            #[rustfmt::skip]
            let expected = charmi_line(&[
                '|' as u32, 15,  0, 0,
                0         , NC, NC, 0,
                0         , NC, NC, 0,
                0         , NC, NC, 0,
                '|' as u32, 15,  0, 0,
            ]);

            let actual = CharmiBuilder::dynamic()
                .add_text("NOTME")
                .build()
                .edit()
                .add_text("|")
                .add_gap(3)
                .add_text("|")
                .build();
            assert_eq!(expected, actual);
        }

        #[test]
        fn add_effect() {
            #[rustfmt::skip]
            let expected = charmi_line(&[
                '|' as u32, 5,  15, 0,
                0         , NC, 10, 0,
                0         , NC, 10, 0,
                '|' as u32, 5,  15, 0,
            ]);

            let actual = CharmiBuilder::dynamic()
                .fg(5)
                .bg(15)
                .add_text("|")
                .no_fg()
                .bg(10)
                .add_style_effect(2)
                .fg(5)
                .bg(15)
                .add_text("|")
                .build();
            assert_eq!(expected, actual);

            let actual = CharmiBuilder::dynamic()
                .fg(5)
                .bg(15)
                .add_text("||||")
                .build()
                .edit()
                .no_fg()
                .bg(10)
                .skip_cells(1)
                .add_style_effect(2)
                .build();
            assert_eq!(expected, actual);
        }

        #[test]
        fn apply_style() {
            #[rustfmt::skip]
            let expected = charmi_line(&[
                '|' as u32, 5,  15, 0,
                0         , NC, 10, 0,
                0         , NC, 10, 0,
                '|' as u32, 5,  15, 0,
            ]);

            let actual = CharmiBuilder::dynamic()
                .fg(5)
                .bg(15)
                .add_text("|")
                .no_fg()
                .bg(10)
                .apply_style(2)
                .fg(5)
                .bg(15)
                .add_text("|")
                .build();
            assert_eq!(expected, actual);

            let actual = CharmiBuilder::dynamic()
                .gap_char('#')
                .fg(8)
                .bg(8)
                .add_text("|##|")
                .build()
                .edit()
                .fg(5)
                .bg(15)
                .apply_style(1)
                .no_fg()
                .bg(10)
                .add_style_effect(2)
                .fg(5)
                .bg(15)
                .apply_style(1)
                .build();
            assert_eq!(expected, actual);
        }

        #[test]
        fn add_text_wrap_chars() {
            println!("Case 1: Basic wrapping across multiple lines");
            #[rustfmt::skip]
            let expected = unsafe {
                CharmiImage::from_raw_u32(3, 3, &[
                    'a' as u32, 15, 0, 0,
                    'b' as u32, 15, 0, 0,
                    'c' as u32, 15, 0, 0,

                    'd' as u32, 15, 0, 0,
                    'e' as u32, 15, 0, 0,
                    'f' as u32, 15, 0, 0,

                    'g' as u32, 15, 0, 0,
                    '!' as u32, 15, 0, 0,
                    ' ' as u32, 15, 0, 0,
                ])
            };
            let actual = CharmiBuilder::fixed_width(3)
                .with_fill(' ')
                .add_text_wrap_chars("abcdefg")
                .add_text("!")
                .build();
            assert_eq!(expected, actual);

            println!("Case 2: Wrapping with double-width chars + splitting");
            #[rustfmt::skip]
            let expected = unsafe {
                CharmiImage::from_raw_u32(4, 3, &[
                    'A' as u32, 15, 0, 0,
                    'バ' as u32, 15, 0, 0,
                    SC, 32, 32, 0,
                    '#' as u32, 15, 0, 0,

                    'カ' as u32, 15, 0, 0,
                    SC, 32, 32, 0,
                    'め' as u32, 15, 0, 0,
                    SC, 32, 32, 0,

                    'の' as u32, 15, 0, 0,
                    SC, 32, 32, 0,
                    '#' as u32, 15, 0, 0,
                    '#' as u32, 15, 0, 0,
                ])
            };
            let actual = CharmiBuilder::fixed_width(4)
                .with_fill('#')
                .add_text_wrap_chars("Aバカめ")
                .add_text("の")
                .build();

            assert_eq!(expected, actual);

            println!("Case 3: Builder has a fixed width of 0");
            let actual = CharmiBuilder::fixed_width(0)
                .add_text_wrap_chars("abcdef")
                .build();
            assert_eq!(CharmiImage::default(), actual);

            println!("Case 4: Width is 1 and there are unicode characters");
            #[rustfmt::skip]
            let expected = unsafe {
                CharmiImage::from_raw_u32(1, 8, &[
                    '<' as u32, 15, 0, 0,
                    '>' as u32, 15, 0, 0,
                    '<' as u32, 15, 0, 0,
                    '>' as u32, 15, 0, 0,
                    'A' as u32, 15, 0, 0,
                    'B' as u32, 15, 0, 0,
                    'C' as u32, 15, 0, 0,
                    'D' as u32, 15, 0, 0,
                ])
            };

            let actual = CharmiBuilder::fixed_width(1)
                .split_chars('<', '>')
                .add_text_wrap_chars("バカABCD")
                .build();

            assert_eq!(expected, actual);
        }

        #[test]
        fn add_text_wrap_chars_hyphenate() {
            println!("Case 1: Basic hyphenation with ASCII characters");
            #[rustfmt::skip]
            let expected = unsafe {
                CharmiImage::from_raw_u32(4, 2, &[
                    'A' as u32, 15, 0, 0,
                    'B' as u32, 15, 0, 0,
                    'C' as u32, 15, 0, 0,
                    '-' as u32, 15, 0, 0,

                    'D' as u32, 15, 0, 0,
                    'E' as u32, 15, 0, 0,
                    'F' as u32, 15, 0, 0,
                    'G' as u32, 15, 0, 0,
                ])
            };
            let actual = CharmiBuilder::fixed_width(4)
                .wrap_hyphenate(true)
                .add_text_wrap_chars("ABCDEFG")
                .build();

            assert_eq!(expected, actual);

            println!("Case 2: Hyphen at the end of the line with double-width chars");
            #[rustfmt::skip]
            let expected = unsafe {
                CharmiImage::from_raw_u32(4, 2, &[
                    'A' as u32, 15, 0, 0,
                    'バ' as u32, 15, 0, 0,
                    SC, 32, 32, 0,
                    '-' as u32, 15, 0, 0,

                    'カ' as u32, 15, 0, 0,
                    SC, 32, 32, 0,
                    'B' as u32, 15, 0, 0,
                    'C' as u32, 15, 0, 0,
                ])
            };
            let actual = CharmiBuilder::fixed_width(4)
                .wrap_hyphenate(true)
                .add_text_wrap_chars("AバカBC")
                .build();

            assert_eq!(expected, actual);

            println!("Case 3: Hyphen with a space fill at the end of the line due to a double width char");
            #[rustfmt::skip]
            let expected = unsafe {
                CharmiImage::from_raw_u32(4, 2, &[
                    'バ' as u32, 15, 0, 0,
                    SC, 32, 32, 0,
                    '-' as u32, 15, 0, 0,
                    '#' as u32, 15, 0, 0,

                    'カ' as u32, 15, 0, 0,
                    SC, 32, 32, 0,
                    'カ' as u32, 15, 0, 0,
                    SC, 32, 32, 0,
                ])
            };
            let actual = CharmiBuilder::fixed_width(4)
                .with_fill('#')
                .wrap_hyphenate(true)
                .add_text_wrap_chars("バカカ")
                .build();

            assert_eq!(expected, actual);

            println!("Case 4: With a width of 1, hyphenation is ignored");
            #[rustfmt::skip]
            let expected = unsafe {
                CharmiImage::from_raw_u32(1, 7, &[
                    'A' as u32, 15, 0, 0,
                    'B' as u32, 15, 0, 0,
                    'C' as u32, 15, 0, 0,
                    'D' as u32, 15, 0, 0,
                    'E' as u32, 15, 0, 0,
                    'F' as u32, 15, 0, 0,
                    'G' as u32, 15, 0, 0,
                ])
            };

            let actual = CharmiBuilder::fixed_width(1)
                .wrap_hyphenate(true)
                .add_text_wrap_chars("ABCDEFG")
                .build();
            assert_eq!(expected, actual);
        }

        #[test]
        fn add_text_wrap_line() {
            println!("Case 1: No wrapping needed");
            #[rustfmt::skip]
            let expected = charmi_line(&[
                'A' as u32, 15, 0, 0,
                'B' as u32, 15, 0, 0,
                'C' as u32, 15, 0, 0,
            ]);

            let actual = CharmiBuilder::fixed_width(3)
                .add_text_wrap_line("ABC")
                .build();
            assert_eq!(expected, actual);

            println!("Case 2: Wrapping is necessary");
            #[rustfmt::skip]
            let expected = unsafe {
                CharmiImage::from_raw_u32(3, 2, &[
                    'A' as u32, 15, 0, 0,
                    'B' as u32, 15, 0, 0,
                    'C' as u32, 15, 0, 0,

                    'D' as u32, 15, 0, 0,
                    'E' as u32, 15, 0, 0,
                    0_u32, NC, NC, 0,
                ])
            };
            let actual = CharmiBuilder::fixed_width(3)
                .add_text_wrap_line("ABCDE")
                .build();
            assert_eq!(expected, actual);

            println!("Case 3: Text is too long for one line (character breaking)");
            #[rustfmt::skip]
            let expected = unsafe {
                CharmiImage::from_raw_u32(3, 3, &[
                    'A' as u32, 15, 0, 0,
                    'B' as u32, 15, 0, 0,
                    'C' as u32, 15, 0, 0,

                    'D' as u32, 15, 0, 0,
                    'E' as u32, 15, 0, 0,
                    'F' as u32, 15, 0, 0,

                    'G' as u32, 15, 0, 0,
                    0_u32, NC, NC, 0,
                    0_u32, NC, NC, 0,
                ])
            };

            let actual = CharmiBuilder::fixed_width(3)
                .add_text_wrap_line("ABCDEFG")
                .build();
            assert_eq!(expected, actual);
        }

        #[test]
        fn add_text_wrap_words() {
            println!("Case 1: Basic wrapping on a fixed width");
            #[rustfmt::skip]
            let expected = charmi_double_line(&[
                'a' as u32, 15,  0, 0,
                ' ' as u32, 15,  0, 0,
                'b' as u32, 15,  0, 0,
                'b' as u32, 15,  0, 0,
                'c' as u32, 15,  0, 0,
                ' ' as u32, 15,  0, 0,
                'd' as u32, 15,  0, 0,
                0         , NC, NC, 0,
            ]);

            let actual = CharmiBuilder::fixed_width(4)
                .add_text_wrap_words(&["a", "bb", "c", "d"])
                .build();
            assert_eq!(expected, actual);

            println!("Case 2: Basic wrapping on a fixed size");
            let actual = CharmiBuilder::fixed_size(4, 2)
                .add_text_wrap_words(&["a", "bb", "c", "d", "e"])
                .build();
            assert_eq!(expected, actual);

            println!("Case 3: Test one word is too long to fit");
            #[rustfmt::skip]
            let expected = unsafe {
                CharmiImage::from_raw_u32(4, 3, &[
                    'a' as u32, 15, 0, 0,
                    ' ' as u32, 15, 0, 0,
                    'b' as u32, 15, 0, 0,
                    'c' as u32, 15, 0, 0,

                    'd' as u32, 15, 0, 0,
                    'e' as u32, 15, 0, 0,
                    'f' as u32, 15, 0, 0,
                    'g' as u32, 15, 0, 0,

                    'h' as u32, 15, 0, 0,
                    'i' as u32, 15, 0, 0,
                    'j' as u32, 15, 0, 0,
                    0_u32, NC, NC, 0,
                ])
            };

            let actual = CharmiBuilder::fixed_width(4)
                .add_text_wrap_words(&["a", "bcdefghij"])
                .build();

            assert_eq!(expected, actual);
        }

        #[test]
        fn add_text_wrap_words_no_spacing() {
            #[rustfmt::skip]
            let expected = charmi_double_line(&[
                'a' as u32, 15,  0, 0,
                ' ' as u32, 15,  0, 0,
                'b' as u32, 15,  0, 0,
                0         , NC, NC, 0,
                'c' as u32, 15,  0, 0,
                'c' as u32, 15,  0, 0,
                'd' as u32, 15,  0, 0,
                0         , NC, NC, 0,
            ]);

            let actual = CharmiBuilder::fixed_width(4)
                .wrap_without_spacing()
                .add_text_wrap_words(&["a", " ", "b", "cc", "d"])
                .build();
            assert_eq!(expected, actual);

            let actual = CharmiBuilder::fixed_width(4)
                .wrap_without_spacing()
                .gap_char('#')
                .add_text_wrap_words(&["a ", "b#", "c", "c", "d#"])
                .build();
            assert_eq!(expected, actual);

            let actual = CharmiBuilder::fixed_size(4, 2)
                .wrap_without_spacing()
                .add_text_wrap_words(&["a", " ", "b", "cc", "d", "XX"])
                .build();
            assert_eq!(expected, actual);

            let actual = CharmiBuilder::fixed_size(4, 2)
                .wrap_without_spacing()
                .gap_char('#')
                .add_text_wrap_words(&["a ", "b#", "c", "c", "d", "XX"])
                .build();
            assert_eq!(expected, actual);
        }
    }

    mod positioning {
        use super::*;

        #[test]
        fn set_cursor() {
            println!("Case 1: Setting cursor position in dynamic mode");
            #[rustfmt::skip]
            let expected = charmi_double_line(&[
                0         , NC, NC, 0,
                'A' as u32, 15,  0, 0,
                'B' as u32, 15,  0, 0,
                0         , NC, NC, 0,
                0         , NC, NC, 0,
                'C' as u32, 15,  0, 0,
                'D' as u32, 15,  0, 0,
                0         , NC, NC, 0,
            ]);

            let actual = CharmiBuilder::dynamic()
                .set_cursor(1, 0)
                .add_text("AB")
                .add_gap(1)
                .set_cursor(1, 1)
                .add_text("CD")
                .build();

            assert_eq!(expected, actual);

            println!("Case 2: Test setting position in fixed width mode");
            let actual = CharmiBuilder::fixed_width(4)
                .set_cursor(1, 0)
                .add_text("AB")
                .set_cursor(1, 1)
                .add_text("CD")
                .build();
            assert_eq!(expected, actual);

            println!("Case 3: Test setting position in fixed size mode");
            let actual = CharmiBuilder::fixed_size(4, 2)
                .set_cursor(1, 0)
                .add_text("AB")
                .set_cursor(1, 1)
                .add_text("CD")
                .build();
            assert_eq!(expected, actual);

            #[rustfmt::skip]
            let expected_bounded = charmi_line(&[
                0         , NC, NC, 0,
                'A' as u32, 15,  0, 0,
                'B' as u32, 15,  0, 0,
                'C' as u32, 15,  0, 0,
            ]);

            let actual = CharmiBuilder::fixed_size(4, 1)
                .set_cursor(1, 0)
                .add_text("ABC")
                .set_cursor(0, 5)
                .add_text("D")
                .build();

            println!("Case 4: Test setting position out of bounds (Y)");
            assert_eq!(expected_bounded, actual);

            let actual = CharmiBuilder::fixed_width(4)
                .set_cursor(1, 0)
                .add_text("ABC")
                .set_cursor(5, 0)
                .add_text("D")
                .build();

            println!("Case 5: Test setting position out of bounds (X)");
            assert_eq!(expected_bounded, actual);
        }

        #[test]
        fn skip_chars() {
            #[rustfmt::skip]
            let expected = charmi_line(&[
                '|' as u32, 15,  0, 0,
                0         , NC, NC, 0,
                0         , NC, NC, 0,
                '-' as u32, 15,  0, 0,
                '|' as u32, 15,  0, 0,
            ]);

            let actual = CharmiBuilder::dynamic()
                .add_text("|")
                .skip_cells(2)
                .add_text("-|")
                .build();
            assert_eq!(expected, actual);

            let actual = CharmiBuilder::fixed_width(5)
                .add_text("-")
                .skip_cells(2)
                .add_text("-")
                .build()
                .edit()
                .add_text("|")
                .skip_cells(3)
                .add_text("|")
                .build();
            assert_eq!(expected, actual);
        }

        #[test]
        fn next_line() {
            #[rustfmt::skip]
            let expected = charmi_double_line(&[
                '|' as u32, 15,  0, 0,
                '|' as u32, 15,  0, 0,
                '|' as u32, 15,  0, 0,
                '-' as u32, 15,  0, 0,
                '-' as u32, 15,  0, 0,
                '-' as u32, 15,  0, 0,
            ]);

            let actual = CharmiBuilder::dynamic()
                .add_text("|||")
                .next_line()
                .add_text("---")
                .build();
            assert_eq!(expected, actual);

            let actual = CharmiBuilder::fixed_width(3)
                .add_text("|||")
                .next_line()
                .add_text("---")
                .build();
            assert_eq!(expected, actual);

            let actual = CharmiBuilder::fixed_size(3, 2)
                .add_text("|||")
                .next_line()
                .add_text("---")
                .next_line()
                .build();
            assert_eq!(expected, actual);

            let actual = CharmiBuilder::fixed_size(3, 2)
                .next_line()
                .add_text("---")
                .build()
                .edit()
                .add_text("|||")
                .build();
            assert_eq!(expected, actual);
        }

        #[test]
        fn add_line() {
            #[rustfmt::skip]
            let expected = unsafe {
                // Manually verified
                CharmiImage::from_raw_u32(1, 3, &[
                    '1' as u32, 15, 0, 0,
                    '2' as u32, 15, 0, 0,
                    '3' as u32, 15, 0, 0,
                ])
            };
            let actual = CharmiBuilder::dynamic()
                .add_line("1")
                .add_line("2")
                .add_line("3") // Final add_line does not create a new line right away
                .build();
            assert_eq!(expected, actual);
        }

        #[test]
        fn skip_lines() {
            #[rustfmt::skip]
            let expected = unsafe {
                // Manually verified
                CharmiImage::from_raw_u32(1, 4, &[
                    '1' as u32, 15,  0, 0,
                    '2' as u32, 15,  0, 0,
                    0         , NC, NC, 0,
                    '3' as u32, 15,  0, 0,
                ])
            };

            let actual = CharmiBuilder::dynamic()
                .add_text("1")
                .skip_lines(1)
                .add_text("2")
                .skip_lines(2)
                .add_text("3")
                .build();
            assert_eq!(expected, actual);

            let actual = CharmiBuilder::fixed_width(1)
                .add_text("1")
                .skip_lines(1)
                .add_text("2")
                .skip_lines(2)
                .add_text("3")
                .build();
            assert_eq!(expected, actual);

            let actual = CharmiBuilder::fixed_size(1, 4)
                .add_text("1")
                .skip_lines(1)
                .add_text("2")
                .skip_lines(2)
                .add_text("3")
                .skip_lines(5)
                .build();
            assert_eq!(expected, actual);
        }
    }
}
