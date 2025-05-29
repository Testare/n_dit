use std::fmt::Debug;
use std::io::Write;
use std::sync::Arc;

use bevy::prelude::*;
use bevy::render::gpu_readback::ReadbackComplete;
use bevy::render::render_resource::ShaderType;

use super::{CharCell, CharmiFill};
use crate::CharmiBuilder;
/// ## Unicode
/// ### Note on cracking
///
/// When double-width/"full-width" characters are in an image, but that image is drawn on, split,
/// or otherwise manipulated in such a way that only half the character can be shown, we call that
/// "cracking."
///
/// For instance, if you had the Japanese greeting "バカめ", but you then wrote the english
/// greeting "Hello" over it. The original phrase is 6 cells wide, but the second one is 5, but
/// unicode doesn't have a way currently of just rendering the second part of the character.
///
/// So we specify how we want this cell to behave when it is broken into two pieces. Internally,
/// double-width characters are stored as two CharCell items anyways: The second CharCell has a
/// `ch` value of `CharmiImage::SUPPRESSED_CHAR`, which is a u32 that is never a valid unicode
/// character. The `fg` and `bg` of this second cell are actually not indicative of fg or bg:
/// Instead they store the `ch` values to use if the preceding double-width character cannot fit
/// (the bg value is the cell if the second half of the )
///
/// For example, in the previous example, let's say the the CharCell in the second and
/// sixth positions had a fg of 'A' (as a u32) and a bg of 'B'. After writing "Hello" over the
/// first 5 cells, the sixth cell would become "A", with the fg and bg values of the previous
/// `め` character ("HelloA"). If Hello was written over the last 5 cells instead, skipping the
/// first cell, it would become "BHello".
///
/// Ideally, the API handles this so you don't need to worry about it much. Just know that
/// if you construct a CharmiImage, you can specify "split_char" for the double-width character
/// to control how they look when the full character won't fit.
///
#[derive(Asset, Clone, Debug, Default, Eq, Ord, PartialEq, PartialOrd, ShaderType, TypePath)]
#[repr(C)]
#[type_path = "charmi"]
pub struct CharmiImage {
    pub(crate) width: u32,
    pub(crate) height: u32,
    #[size(runtime)]
    pub(crate) cells: Arc<[CharCell]>,
}

impl CharmiImage {
    pub const NO_COLOR: u32 = 0xFE << 24;
    pub const TRUE_COLOR: u32 = 0x1 << 24;
    pub const SUPPRESSED_CHAR: u32 = 0xDFFF;

    pub fn cells(&self) -> &[CharCell] {
        self.cells.as_ref()
    }

    pub fn width(&self) -> usize {
        self.width as usize
    }

    pub fn width32(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height as usize
    }

    pub fn height32(&self) -> u32 {
        self.height
    }

    /// # Safety
    /// CharCells must be correctly formed, with suppressed chars after the double-width chars and
    /// cell count must be
    pub unsafe fn from_raw_cells(width: u32, height: u32, cells: Arc<[CharCell]>) -> Self {
        Self {
            width,
            height,
            cells,
        }
    }

    /// # Safety
    /// CharCells must be correctly formed, with suppressed chars after the double-width chars
    pub unsafe fn from_raw_u32(width: u32, height: u32, cells: &[u32]) -> Self {
        Self {
            width,
            height,
            cells: cells
                .chunks(4)
                .map(|c| CharCell {
                    ch: c[0],
                    fg: c[1],
                    bg: c[2],
                    attr: c[3],
                })
                .collect(),
        }
    }

    /// Allows editting a copy of this image
    ///
    /// Does not mutable change the source image
    pub fn edit(&self) -> CharmiBuilder {
        CharmiBuilder::edit(self.clone())
    }

    /// Convenience method for [`CharmiBuilder::dynamic`]
    pub fn build_dynamic() -> CharmiBuilder {
        CharmiBuilder::dynamic()
    }

    /// Convenience method for [`CharmiBuilder::fixed_width`]
    pub fn build_fixed_width(width: u32) -> CharmiBuilder {
        CharmiBuilder::fixed_width(width)
    }

    /// Convenience method for [`CharmiBuilder::fixed_size`]
    pub fn build_sized(width: u32, height: u32) -> CharmiBuilder {
        CharmiBuilder::fixed_size(width, height)
    }

    pub fn new_empty(width: u32, height: u32) -> Self {
        Self::new_fill(width, height, CharCell::GAP)
    }

    pub fn new_blank(width: u32, height: u32) -> Self {
        Self::new_fill(width, height, CharCell::BLANK)
    }

    pub fn new_fill<F: CharmiFill>(width: u32, height: u32, fill: F) -> Self {
        if width == 0 || height == 0 {
            Self {
                width: 0,
                height: 0,
                cells: Arc::default(),
            }
        } else {
            Self {
                width,
                height,
                cells: vec![fill.as_default_fill(); (width * height) as usize].into(),
            }
        }
    }

    pub fn clip<F: CharmiFill>(&self, x: i32, y: i32, width: u32, height: u32, fill: F) -> Self {
        if x >= self.width as i32
            || y >= self.height as i32
            || x <= -(width as i32)
            || y <= -(height as i32)
            || width == 0
            || height == 0
        {
            return Self::new_fill(width, height, fill);
        }

        let left_padding = (-x).max(0) as usize;
        let right_padding = (x + width as i32 - self.width as i32).max(0) as usize;
        let top_padding = (-y).max(0) as usize;
        let bottom_padding = (y + height as i32 - self.height as i32).max(0) as usize;

        let x_start = x.max(0) as usize;
        let y_start = y.max(0) as usize;

        // The guard should prevent these values from being less than 0
        let x_end = (x + width as i32).min(self.width as i32) as usize;
        let y_end = (y + height as i32).min(self.height as i32) as usize;

        let mut cells: Vec<CharCell> = Vec::with_capacity((width * height) as usize);
        cells.extend(std::iter::repeat_n(
            fill.as_default_fill(),
            top_padding * width as usize,
        ));

        // TODO check if having these loops separate is better performance
        let row_offset = y_start * (self.width as usize);
        let mut row_start = row_offset + x_start;
        let mut row_end = row_offset + x_end;
        for _ in y_start..y_end {
            cells.extend(std::iter::repeat_n(fill.as_default_fill(), left_padding));
            cells.extend(&self.cells[row_start..row_end]);
            row_start += self.width as usize;
            row_end += self.width as usize;
            cells.extend(std::iter::repeat_n(fill.as_default_fill(), right_padding));
        }
        cells.extend(std::iter::repeat_n(
            fill.as_default_fill(),
            bottom_padding * width as usize,
        ));

        if x_start > 0 {
            for (dst_y, y) in (top_padding..).zip(y_start..y_end) {
                let row_offset = y * (self.width as usize);
                let row_start = row_offset + x_start;
                if self.cells[row_start].ch == Self::SUPPRESSED_CHAR {
                    // Crack front/right
                    let dst_row_start = dst_y * (width as usize) + left_padding;
                    cells[dst_row_start] = CharCell {
                        ch: self.cells[row_start].fg,
                        ..self.cells[row_start - 1]
                    }
                }
            }
        }

        if x_end < self.width as usize {
            let dst_offset = left_padding + x_end - x_start - 1;
            for (dst_y, y) in (top_padding..).zip(y_start..y_end) {
                let row_offset = y * (self.width as usize);
                let row_end = row_offset + x_end;
                if self.cells[row_end].ch == Self::SUPPRESSED_CHAR {
                    // Crack back/left
                    let dst_row_end = dst_y * (width as usize) + dst_offset;
                    cells[dst_row_end] = CharCell {
                        ch: self.cells[row_end].bg,
                        ..self.cells[row_end - 1]
                    }
                }
            }
        }

        CharmiImage {
            width,
            height,
            cells: cells.into(),
        }
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
    pub fn clip_intersect(&self, x: u32, y: u32, width: u32, height: u32) -> Self {
        if x >= self.width || y >= self.height || width == 0 || height == 0 {
            return default();
        }

        let new_width = (self.width - x).min(width) as usize;
        let x_start = x as usize;
        let x_end = new_width + x_start;
        let new_height = (self.height - y).min(height) as usize;
        let y_start = y as usize;
        let y_end = new_height + y_start;
        let mut cells: Vec<CharCell> = Vec::with_capacity(new_width * new_height);

        // TODO check if having these loops separate is better performance
        let row_offset = y_start * (self.width as usize);
        let mut row_start = row_offset + x_start;
        let mut row_end = row_offset + x_end;
        for _ in y_start..y_end {
            cells.extend(&self.cells[row_start..row_end]);
            row_start += self.width as usize;
            row_end += self.width as usize;
        }

        if x_start > 0 {
            for (dst_y, y) in (y_start..y_end).enumerate() {
                let row_offset = y * (self.width as usize);
                let row_start = row_offset + x_start;
                if self.cells[row_start].ch == Self::SUPPRESSED_CHAR {
                    // Crack front/right
                    cells[dst_y * new_width] = CharCell {
                        ch: self.cells[row_start].fg,
                        ..self.cells[row_start - 1]
                    }
                }
            }
        }

        if x_end < self.width as usize {
            for (dst_y, y) in (y_start..y_end).enumerate() {
                let row_offset = y * (self.width as usize);
                let row_end = row_offset + x_end;
                let dst_offset = new_width - 1;
                if self.cells[row_end].ch == Self::SUPPRESSED_CHAR {
                    // Crack back/left
                    cells[dst_y * new_width + dst_offset] = CharCell {
                        ch: self.cells[row_end].bg,
                        ..self.cells[row_end - 1]
                    }
                }
            }
        }

        CharmiImage {
            width: new_width as u32,
            height: new_height as u32,
            cells: cells.into(),
        }
    }

    pub fn resize<F: CharmiFill>(&self, width: u32, height: u32, fill: F) -> CharmiImage {
        self.clip(0, 0, width, height, fill)
    }

    pub fn write_out_ansi<W: Write>(
        &self,
        mut out: W,
        cache: Option<&CharmiImage>,
    ) -> std::io::Result<()> {
        use crossterm::queue;
        use crossterm::style::*;
        let initial_size = self.width * self.height * 21;
        let mut buffer = Vec::with_capacity(initial_size as usize);
        let cache_lines: Vec<_> = cache
            .iter()
            .filter(|cache_image| cache_image.width == self.width)
            .flat_map(|cache_image| cache_image.cells.chunks(cache_image.width as usize))
            .collect();
        for (i, line) in self.cells.chunks(self.width as usize).enumerate() {
            if cache_lines.get(i) == Some(&line) {
                continue;
            }
            let mut last_bg = None;
            let mut last_fg = None;
            queue!(
                buffer,
                crossterm::cursor::MoveTo(0, i as u16),
                crossterm::terminal::Clear(crossterm::terminal::ClearType::CurrentLine)
            )?;
            for datum in line.iter().take((self.width * self.height) as usize) {
                // TODO BEFOREMERGE make sure view starts initialized with spaces, not empty
                if datum.ch == 0 || datum.ch == Self::SUPPRESSED_CHAR {
                    continue;
                }
                let Some(charmi_ch) = <char>::from_u32(datum.ch) else {
                    continue;
                };

                if last_fg == Some(datum.fg) {
                } else if datum.fg == CharmiImage::NO_COLOR {
                    // No Foreground, default to white
                    queue!(buffer, SetForegroundColor(crossterm::style::Color::White))?;
                } else if datum.fg > CharmiImage::TRUE_COLOR {
                    last_fg = Some(datum.fg);
                    let r = ((datum.fg >> 16) & 255) as u8;
                    let g = ((datum.fg >> 8) & 255) as u8;
                    let b = (datum.fg & 255) as u8;

                    queue!(
                        buffer,
                        SetForegroundColor(crossterm::style::Color::Rgb { r, g, b })
                    )?;
                } else {
                    last_fg = Some(datum.fg);
                    queue!(
                        buffer,
                        SetForegroundColor(crossterm::style::Color::AnsiValue(datum.fg as u8))
                    )?;
                };

                if last_bg == Some(datum.bg) {
                } else if datum.bg == CharmiImage::NO_COLOR {
                    // No color, default to black
                    queue!(buffer, SetForegroundColor(crossterm::style::Color::Black))?;
                } else if datum.bg >= CharmiImage::TRUE_COLOR {
                    last_bg = Some(datum.bg);
                    let r = ((datum.bg >> 16) & 255) as u8;
                    let g = ((datum.bg >> 8) & 255) as u8;
                    let b = (datum.bg & 255) as u8;
                    queue!(
                        buffer,
                        SetBackgroundColor(crossterm::style::Color::Rgb { r, g, b })
                    )?;
                } else {
                    last_bg = Some(datum.bg);
                    queue!(
                        buffer,
                        SetBackgroundColor(crossterm::style::Color::AnsiValue(datum.bg as u8))
                    )?;
                };
                queue!(buffer, Print(charmi_ch))?;
            }
            queue!(buffer, ResetColor)?;
        }
        buffer.flush()?;
        out.write_all(&buffer)?;
        out.flush()?;
        Ok(())
    }
}

impl From<&ReadbackComplete> for CharmiImage {
    fn from(value: &ReadbackComplete) -> Self {
        #[derive(Debug, Default, ShaderType)]
        #[repr(C)]
        struct CharmiImageVec {
            width: u32,
            height: u32,
            #[size(runtime)]
            cells: Vec<CharCell>,
        }

        let CharmiImageVec {
            width,
            height,
            cells,
        } = value.to_shader_type();
        CharmiImage {
            width,
            height,
            cells: cells.into(),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::CharmiBuilder;

    #[test]
    fn clip_resize_bigger() {
        let test_image = CharmiBuilder::dynamic()
            .no_fg()
            .no_bg()
            .add_text("a")
            .build();

        let expected = CharmiBuilder::dynamic()
            .no_fg()
            .no_bg()
            .add_line("!!!")
            .add_line("!a!")
            .add_text("!!!")
            .build();

        assert_eq!(expected, test_image.clip(-1, -1, 3, 3, '!'));
    }

    #[test]
    fn clip_resize_smaller() {
        let test_image = CharmiBuilder::dynamic()
            .add_line("abcd")
            .add_line("efgh")
            .add_line("ijkl")
            .add_text("mnop")
            .build();

        let expected = CharmiBuilder::dynamic()
            .add_line("fg")
            .add_text("jk")
            .build();

        assert_eq!(expected, test_image.clip(1, 1, 2, 2, None));
    }

    #[test]
    fn clip_double_width() {
        let test_image = CharmiBuilder::dynamic()
            .split_chars('<', '>')
            .add_line("-バ")
            .add_line("カ-")
            .add_line("-バ")
            .add_line("-バ")
            .add_text("カ-")
            .build();

        let expected = CharmiBuilder::dynamic()
            .next_line()
            .add_line("<")
            .add_line(">")
            .add_line("<")
            .add_line("<")
            .add_text(">")
            .build();

        assert_eq!(expected, test_image.clip(1, -1, 1, 6, None));
    }

    #[test]
    fn clip_intersect() {
        let test_image = CharmiBuilder::dynamic()
            .add_line("abc")
            .add_line("def")
            .add_text("ghi")
            .build();

        let expected = CharmiBuilder::dynamic()
            .add_line("ef")
            .add_text("hi")
            .build();

        assert_eq!(expected, test_image.clip_intersect(1, 1, 2, 2));
    }

    #[test]
    fn clip_intersect_double_width() {
        let test_image = CharmiBuilder::dynamic()
            .split_chars('<', '>')
            .add_line("-バ")
            .add_text("カ-")
            .build();

        let expected = CharmiBuilder::dynamic().add_line("<").add_text(">").build();

        assert_eq!(expected, test_image.clip_intersect(1, 0, 1, 2));
    }
}
