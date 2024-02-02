use std::ops::RangeInclusive;

use crossterm::style::ContentStyle;

use super::PlayerUiQItem;
use crate::configuration::{DrawConfiguration, DrawType};
use crate::node_ui::NodeCursor;
use crate::prelude::*;

const INTERSECTION_CHAR: [char; 16] = [
    ' ', '?', '?', '└', '?', '│', '┌', '├', '?', '┘', '─', '┴', '┐', '┤', '┬', '┼',
];

const ARROWS: [&str; 6] = ["↑↑", "→", "↓↓", "←", "↑", "↓"];

#[derive(PartialEq, Eq)]
pub enum BorderType {
    Borderless = 0,
    Bordered = 1,
    Linked = 2,
}

impl BorderType {
    pub fn of(lhs: usize, rhs: usize) -> BorderType {
        if lhs != rhs {
            BorderType::Bordered
        } else {
            match lhs {
                0 => BorderType::Borderless,
                1 => BorderType::Bordered,
                _ => BorderType::Linked,
            }
        }
    }

    pub fn horizontal_border(&self, draw_config: &DrawConfiguration) -> &'static str {
        match self {
            BorderType::Borderless => "  ",
            BorderType::Bordered => "──",
            BorderType::Linked => match draw_config.border_appearance() {
                DrawType::BorderlessLink => "  ",
                DrawType::CrossLink1 => "╫─",
                DrawType::CrossLink2 => "┤├",
                DrawType::CrossLink3 => "┼┼",
                DrawType::DotLink => "..",
            },
        }
    }

    pub fn vertical_border(&self, draw_config: &DrawConfiguration) -> char {
        match self {
            BorderType::Borderless => ' ',
            BorderType::Bordered => '│',
            BorderType::Linked => match draw_config.border_appearance() {
                DrawType::BorderlessLink => ' ',
                DrawType::DotLink => '.',
                _ => '╪',
            },
        }
    }
}

#[allow(clippy::collapsible_else_if)] // Visual symmetry for hover
pub fn border_style_for(
    player_q: &PlayerUiQItem,
    hover_point: &Option<UVec2>,
    action_hover: bool,
    draw_config: &DrawConfiguration,
    x_range: &RangeInclusive<usize>, // TODO usize -> u32?
    y_range: &RangeInclusive<usize>, // TODO include if this border space is empty
) -> ContentStyle {
    let color_scheme = draw_config.color_scheme();

    let NodeCursor(UVec2 {
        x: cursor_x,
        y: cursor_y,
    }) = player_q.node_cursor;

    let points_in_range = points_in_range(x_range, y_range);

    let under_hover = hover_point
        .map(|pt| x_range.contains(&(pt.x as usize)) && y_range.contains(&(pt.y as usize)))
        .unwrap_or(false);

    if under_hover && action_hover {
        color_scheme.immediate_movement()
    } else if !player_q.cursor_is_hidden
        && x_range.contains(&(*cursor_x as usize))
        && y_range.contains(&(*cursor_y as usize))
    {
        if under_hover {
            color_scheme.selected_square_border_hover()
        } else {
            color_scheme.selected_square_border()
        }
    } else if !player_q.available_moves.is_empty()
        && points_in_range
            .iter()
            .any(|pt| player_q.available_moves.contains_key(pt))
    {
        if under_hover {
            color_scheme.possible_movement_hover()
        } else {
            color_scheme.possible_movement()
        }
    } else if !player_q.available_action_targets.is_empty()
        && !points_in_range.is_disjoint(player_q.available_action_targets)
    {
        if under_hover {
            color_scheme.attack_action_hover()
        } else {
            color_scheme.attack_action()
        }
    } else {
        if under_hover {
            color_scheme.grid_border_hover()
        } else {
            color_scheme.grid_border()
        }
    }
}

pub fn arrow_border(compass: Compass, half_border: bool) -> &'static str {
    match compass {
        Compass::North if half_border => ARROWS[4],
        Compass::North => ARROWS[0],
        Compass::East => ARROWS[1],
        Compass::South if half_border => ARROWS[5],
        Compass::South => ARROWS[2],
        Compass::West => ARROWS[3],
    }
}

pub fn intersection_for_pivot(
    left: &[usize; 2],
    right: &[usize; 2],
    draw_config: &DrawConfiguration,
) -> char {
    #[inline]
    fn border_type_bit(config: &DrawConfiguration, one: usize, the_other: usize) -> usize {
        if config.border_appearance() == DrawType::BorderlessLink {
            usize::from(BorderType::of(one, the_other) == BorderType::Bordered)
        } else {
            usize::from(BorderType::of(one, the_other) != BorderType::Borderless)
        }
    }
    let north = border_type_bit(draw_config, left[0], right[0]);
    let east = border_type_bit(draw_config, right[0], right[1]) << 1;
    let south = border_type_bit(draw_config, left[1], right[1]) << 2;
    let west = border_type_bit(draw_config, left[0], left[1]) << 3;

    INTERSECTION_CHAR[north | east | south | west]
}

fn points_in_range(
    x_range: &RangeInclusive<usize>,
    y_range: &RangeInclusive<usize>,
) -> HashSet<UVec2> {
    let mut set = HashSet::default();
    for x in x_range.clone() {
        for y in y_range.clone() {
            set.insert(UVec2 {
                x: x as u32,
                y: y as u32,
            });
        }
    }
    set
}
