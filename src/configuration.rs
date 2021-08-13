#[derive(Clone)]
pub struct DrawConfiguration {
    half_char: char,
    draw_type: DrawType,
    fill_method: FillMethod,
}

impl DrawConfiguration {
    pub fn half_char(&self) -> char {
        self.half_char
    }

    pub fn border_appearance(&self) -> DrawType {
        self.draw_type
    }

    pub fn tail_appearance(&self) -> FillMethod {
        self.fill_method
    }
}

impl Default for DrawConfiguration {
    fn default() -> Self {
        DrawConfiguration {
            half_char: '~',
            draw_type: DrawType::CrossLink2,
            fill_method: FillMethod::Brackets,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum DrawType {
    BorderlessLink = 0,
    CrossLink1,
    CrossLink2,
    CrossLink3,
    DotLink,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum FillMethod {
    Brackets = 0,
    NoFill = 1,
    HeadCopy = 2,
    DotFill = 3,
    Sequence = 4,
}
