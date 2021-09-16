use crossterm::style::{Attribute, Color, StyledContent, Stylize};
use getset::{CopyGetters, Getters};
use std::fmt::Display;

#[derive(Clone, Debug, CopyGetters, Getters)]
pub struct DrawConfiguration {
    #[get_copy = "pub"]
    border_appearance: DrawType,
    #[get = "pub"]
    color_scheme: ColorScheme,
    #[get_copy = "pub"]
    half_char: char,
    #[get_copy = "pub"]
    tail_appearance: FillMethod,
}

#[derive(Copy, Clone, Debug, Default)]
pub struct UiFormat(Option<Color>, Option<Color>, Option<Attribute>);

#[derive(Clone, CopyGetters, Debug)]
pub struct ColorScheme {
    #[get_copy = "pub"]
    access_point: UiFormat,
    #[get_copy = "pub"]
    attack_action: UiFormat,
    #[get_copy = "pub"]
    enemy_team: UiFormat,
    #[get_copy = "pub"]
    grid_border_default: UiFormat,
    #[get_copy = "pub"]
    mon: UiFormat,
    #[get_copy = "pub"]
    player_team: UiFormat,
    #[get_copy = "pub"]
    possible_movement: UiFormat,
    #[get_copy = "pub"]
    selected_menu_item: UiFormat,
    #[get_copy = "pub"]
    selected_square: UiFormat,
    #[get_copy = "pub"]
    selected_square_border: UiFormat,
    #[get_copy = "pub"]
    player_team_active: UiFormat,
    #[get_copy = "pub"]
    player_team_tapped: UiFormat,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DrawType {
    BorderlessLink = 0,
    CrossLink1,
    CrossLink2, // Personal favorite
    CrossLink3,
    DotLink,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FillMethod {
    Brackets = 0, // Personal favorite
    NoFill = 1,   // Really terrible
    HeadCopy = 2, // Hard to tell where head is
    DotFill = 3, // Kinda works with DotLink, but not perfectly. Might need to adjust color scheme logic
    Sequence = 4, // Nice additional information, but a little rough on the eyes
}

impl ColorScheme {
    pub const CLASSIC: Self = ColorScheme {
        access_point: UiFormat::new(
            Some(Color::Black),
            Some(Color::Green),
            Some(Attribute::Underlined),
        ),
        attack_action: UiFormat::new(Some(Color::White), Some(Color::Red), None),
        mon: UiFormat::new(Some(Color::Yellow), None, Some(Attribute::Bold)),
        selected_menu_item: UiFormat::new(None, None, Some(Attribute::Reverse)),
        selected_square: UiFormat::new(None, None, Some(Attribute::Reverse)),
        selected_square_border: UiFormat::new(Some(Color::White), Some(Color::DarkGrey), None),
        grid_border_default: UiFormat::new(Some(Color::Green), None, None),
        possible_movement: UiFormat::new(Some(Color::White), Some(Color::DarkGrey), None),
        player_team: UiFormat::new(Some(Color::Blue), None, None),
        enemy_team: UiFormat::new(Some(Color::Red), None, None),
        player_team_active: UiFormat::new(
            Some(Color::White),
            Some(Color::Blue),
            Some(Attribute::Bold),
        ),
        player_team_tapped: UiFormat::new(Some(Color::Grey), None, None),
    };

    pub const MODERN: Self = ColorScheme {
        access_point: UiFormat::new(
            Some(Color::Black),
            Some(Color::Green),
            Some(Attribute::SlowBlink),
        ),
        mon: UiFormat::new(Some(Color::Yellow), None, None),
        selected_menu_item: UiFormat::new(None, None, Some(Attribute::Reverse)),
        selected_square: UiFormat::new(None, None, Some(Attribute::Reverse)),
        selected_square_border: UiFormat::new(Some(Color::White), Some(Color::DarkGrey), None),
        grid_border_default: UiFormat::new(Some(Color::Green), None, None),
        possible_movement: UiFormat::new(Some(Color::White), Some(Color::DarkGrey), None),

        // TODO Separate color schemes
        enemy_team: UiFormat::new(Some(Color::Red), None, None),
        player_team: UiFormat::new(Some(Color::AnsiValue(214)), None, None),
        player_team_active: UiFormat::new(Some(Color::White), Some(Color::Blue), None),
        player_team_tapped: UiFormat::new(Some(Color::DarkBlue), Some(Color::Blue), None),
        ..Self::CLASSIC
    };
}

impl Default for ColorScheme {
    fn default() -> Self {
        Self::CLASSIC
    }
}

impl UiFormat {
    pub const NONE: Self = UiFormat(None, None, None);

    const fn new(fg: Option<Color>, bg: Option<Color>, attr: Option<Attribute>) -> Self {
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

impl Default for DrawConfiguration {
    fn default() -> Self {
        DrawConfiguration {
            color_scheme: ColorScheme::default(),
            border_appearance: DrawType::CrossLink2,
            tail_appearance: FillMethod::Brackets,
            half_char: '~',
        }
    }
}
