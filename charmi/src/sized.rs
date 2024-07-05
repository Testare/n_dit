use super::CharmiCell;
use crate::ColorValue;

#[derive(Clone, Debug)]
pub struct CharmiSized {
    grid: Vec<CharmiCell>,
    width: usize,
    height: usize,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct CharmiStyle {
    fg: Option<ColorValue>,
    bg: Option<ColorValue>,
}

impl CharmiSized {
    fn new_empty(width: usize, height: usize) -> Self {
        Self::new_fill(width, height, CharmiCell::new_empty())
    }

    fn new_blank(width: usize, height: usize) -> Self {
        Self::new_fill(width, height, CharmiCell::new_blank())
    }

    fn new_fill(width: usize, height: usize, cell: CharmiCell) -> Self {
        Self {
            grid: vec![cell; width * height],
            height,
            width,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct CharmiString(Vec<CharmiCell>);

impl CharmiString {
    fn new() -> Self {
        Self::default()
    }

    fn builder() -> CharmiStringBuilder {
        CharmiStringBuilder(Default::default(), Default::default())
    }

    fn map_cell<B, F: Fn(CharmiCell) -> CharmiCell>(&self, f: F) -> Self {
        Self(self.0.iter().copied().map(f).collect())
    }

    fn for_each_cell<B, F: Fn(&mut CharmiCell)>(&mut self, f: F) {
        for cell in self.0.iter_mut() {
            f(cell);
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct CharmiStringBuilder(CharmiString, CharmiStyle);

impl CharmiStringBuilder {
    const fn fg(mut self, color: Option<ColorValue>) -> Self {
        self.1.fg = color;
        self
    }

    const fn bg(mut self, color: Option<ColorValue>) -> Self {
        self.1.fg = color;
        self
    }

    fn append<S: AsRef<str>>(mut self, s: S) -> Self {
        // Optimize vec allocation
        for character in s.as_ref().chars() {
            // unicode_width
            self.0 .0.push(CharmiCell {
                character: Some(character),
                fg: self.1.fg,
                bg: self.1.bg,
            });
        }
        self
    }

    fn build(self) -> CharmiString {
        let CharmiStringBuilder(a, _) = self;
        a
    }
}
