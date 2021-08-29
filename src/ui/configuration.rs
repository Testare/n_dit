use crossterm::style::{Attribute, Color, StyledContent, Stylize};
use std::fmt::Display;
#[derive(Clone, Debug)]
pub struct DrawConfiguration {
    color_scheme: ColorScheme,
    draw_type: DrawType,
    fill_method: FillMethod,
    half_char: char,
}

#[derive(Copy, Clone, Debug)]
pub struct UiFormat(Option<Color>, Option<Color>, Option<Attribute>);

#[derive(Clone, Debug)]
pub struct ColorScheme {
    access_point: UiFormat,
    mon: UiFormat,
    selected_square: UiFormat,
    selected_square_border: UiFormat,
    grid_border_default: UiFormat,
    pub possible_movement: UiFormat,
    pub friendly_team: UiFormat,
    pub enemy_team: UiFormat,
}

impl ColorScheme {
    pub fn selected_square(&self) -> UiFormat {
        self.selected_square
    }

    pub fn selected_square_border(&self) -> UiFormat {
        self.selected_square_border
    }

    pub fn grid_border_default(&self) -> UiFormat {
        self.grid_border_default
    }

    pub fn mon(&self) -> UiFormat {
        self.mon
    }

    pub fn access_point(&self) -> UiFormat {
        self.access_point
    }

    pub fn player_team(&self) -> UiFormat {
        self.friendly_team
    }

    pub fn enemy_team(&self) -> UiFormat {
        self.enemy_team
    }
}

impl Default for ColorScheme {
    fn default() -> Self {
        ColorScheme {
            access_point: UiFormat::new(Some(Color::Black), Some(Color::Green), None),
            mon: UiFormat::new(Some(Color::Yellow), None, None),
            selected_square: UiFormat::new(None, None, Some(Attribute::Reverse)),
            selected_square_border: UiFormat::new(Some(Color::White), Some(Color::DarkGrey), None),
            grid_border_default: UiFormat::new(Some(Color::Green), None, None),
            possible_movement: UiFormat::default(),
            friendly_team: UiFormat::new(Some(Color::Blue), None, None),
            enemy_team: UiFormat::new(Some(Color::Red), None, None),
        }
    }
}

impl UiFormat {
    pub const NONE: Self = UiFormat(None, None, None);

    fn new(fg: Option<Color>, bg: Option<Color>, attr: Option<Attribute>) -> Self {
        UiFormat(fg, bg, attr)
    }

    pub fn apply<D: Display, S: Stylize<Styled = StyledContent<D>>>(&self, s: S) -> String {
        let mut styled = s.stylize();
        if let Some(fg) = self.0 {
            styled = styled.with(fg);
        }
        if let Some(bg) = self.1 {
            styled = styled.on(bg);
        }
        if let Some(attr) = self.2 {
            styled = styled.attribute(attr);
        }
        styled.to_string()
    }
}

impl Default for UiFormat {
    fn default() -> Self {
        UiFormat::new(None, None, None)
    }
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

    pub fn color_scheme(&self) -> &ColorScheme {
        &self.color_scheme
    }
}

impl Default for DrawConfiguration {
    fn default() -> Self {
        DrawConfiguration {
            color_scheme: ColorScheme::default(),
            draw_type: DrawType::CrossLink2,
            fill_method: FillMethod::Brackets,
            half_char: '~',
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DrawType {
    BorderlessLink = 0,
    CrossLink1,
    CrossLink2,
    CrossLink3,
    DotLink,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FillMethod {
    Brackets = 0,
    NoFill = 1,
    HeadCopy = 2,
    DotFill = 3,
    Sequence = 4,
}
