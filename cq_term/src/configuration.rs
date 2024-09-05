use crossterm::style::{Attribute, Attributes, Color, ContentStyle};
use getset::{CopyGetters, Getters};

use crate::prelude::*;

#[derive(Clone, Debug, CopyGetters, Getters, Resource)]
pub struct DrawConfiguration {
    #[get_copy = "pub"]
    border_appearance: DrawType,
    #[get = "pub"]
    color_scheme: ColorScheme,
    #[get_copy = "pub"]
    half_char: char,
}

// TODO Each of these is complex object that contains no-hover and hover variants
#[derive(Clone, CopyGetters, Debug)]
#[get_copy = "pub"]
pub struct ColorScheme {
    access_point: ContentStyle,
    attack_action: ContentStyle,
    attack_action_hover: ContentStyle,
    context_menu: ContentStyle,
    context_menu_item: ContentStyle,
    context_menu_item_hover: ContentStyle,
    grid_border: ContentStyle,
    grid_border_hover: ContentStyle,
    possible_movement: ContentStyle,
    possible_movement_hover: ContentStyle,
    immediate_movement: ContentStyle,
    selected_square: ContentStyle,
    selected_square_border: ContentStyle,
    selected_square_border_hover: ContentStyle,
    shop_ui_buy_button: ContentStyle,
    shop_ui_done_button: ContentStyle,
    shop_ui_listing_item: ContentStyle,
    shop_ui_listing_item_selected: ContentStyle,
    player_team_active: ContentStyle,
    player_team_tapped: ContentStyle,
    menu_hover: ContentStyle,
    menu_title: ContentStyle,
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
            attack_action_hover: style(Some(Color::DarkBlue), Some(Color::Red), None),
            context_menu: style(Some(Color::DarkBlue), None, None),
            context_menu_item: style(Some(Color::Yellow), None, None),
            context_menu_item_hover: style(Some(Color::Yellow), Some(Color::DarkBlue), None),
            selected_square: style(None, None, Some(Attribute::Reverse)),
            selected_square_border: style(Some(Color::White), Some(Color::DarkGrey), None),
            selected_square_border_hover: style(Some(Color::Blue), Some(Color::DarkGrey), None),
            shop_ui_buy_button: style(
                Some(Color::White),
                Some(Color::Rgb {
                    r: 55,
                    g: 105,
                    b: 65,
                }),
                None,
            ),
            shop_ui_done_button: style(Some(Color::White), Some(Color::DarkBlue), None),
            shop_ui_listing_item: style(None, None, None),
            shop_ui_listing_item_selected: style(Some(Color::Red), None, None),
            grid_border: style(Some(Color::Green), None, None),
            grid_border_hover: style(Some(Color::Blue), None, None),
            possible_movement: style(Some(Color::White), Some(Color::DarkGrey), None),
            possible_movement_hover: style(Some(Color::Blue), Some(Color::DarkGrey), None),
            immediate_movement: style(Some(Color::Yellow), Some(Color::DarkGrey), None),
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
