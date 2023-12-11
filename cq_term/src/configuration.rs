use bevy::prelude::Resource;
use crossterm::style::{Attribute, Attributes, Color, ContentStyle};
use getset::{CopyGetters, Getters};

#[derive(Clone, Debug, CopyGetters, Getters, Resource)]
pub struct DrawConfiguration {
    #[get_copy = "pub"]
    border_appearance: DrawType,
    #[get = "pub"]
    color_scheme: ColorScheme,
    #[get_copy = "pub"]
    half_char: char,
}

#[derive(Clone, CopyGetters, Debug)]
pub struct ColorScheme {
    #[get_copy = "pub"]
    access_point: ContentStyle,
    #[get_copy = "pub"]
    attack_action: ContentStyle,
    #[get_copy = "pub"]
    grid_border_default: ContentStyle,
    #[get_copy = "pub"]
    possible_movement: ContentStyle,
    #[get_copy = "pub"]
    selected_square: ContentStyle,
    #[get_copy = "pub"]
    selected_square_border: ContentStyle,
    #[get_copy = "pub"]
    player_team_active: ContentStyle,
    #[get_copy = "pub"]
    player_team_tapped: ContentStyle,
    #[get_copy = "pub"]
    menu_hover: ContentStyle,
    #[get_copy = "pub"]
    menu_title: ContentStyle,
    #[get_copy = "pub"]
    menu_title_hover: ContentStyle,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DrawType {
    BorderlessLink = 0,
    CrossLink1,
    CrossLink2, // Personal favorite
    CrossLink3,
    DotLink,
}

impl Default for ColorScheme {
    fn default() -> Self {
        ColorScheme {
            access_point: style(
                Some(Color::Black),
                Some(Color::Green),
                Some(Attribute::Underlined),
            ),
            attack_action: style(Some(Color::White), Some(Color::Red), None),
            selected_square: style(None, None, Some(Attribute::Reverse)),
            selected_square_border: style(Some(Color::White), Some(Color::DarkGrey), None),
            grid_border_default: style(Some(Color::Green), None, None),
            possible_movement: style(Some(Color::White), Some(Color::DarkGrey), None),
            player_team_active: style(
                Some(Color::Black),
                Some(Color::White),
                Some(Attribute::Bold),
            ),
            player_team_tapped: style(Some(Color::Grey), None, None),
            menu_hover: style(Some(Color::Blue), None, None),
            menu_title: style(None, None, None),
            menu_title_hover: style(None, None, Some(Attribute::Reverse)),
        }
    }
}

pub fn style(fg: Option<Color>, bg: Option<Color>, attr: Option<Attribute>) -> ContentStyle {
    let attributes = attr
        .map(|attr| Attributes::default() | attr)
        .unwrap_or_default();
    ContentStyle {
        foreground_color: fg,
        background_color: bg,
        attributes,
    }
}

impl Default for DrawConfiguration {
    fn default() -> Self {
        DrawConfiguration {
            color_scheme: ColorScheme::default(),
            border_appearance: DrawType::CrossLink2,
            half_char: '~',
        }
    }
}
