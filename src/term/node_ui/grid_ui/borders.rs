use std::ops::RangeInclusive;

use crate::term::configuration::{DrawConfiguration, DrawType, UiFormat};
use crate::term::node_ui::{NodeCursor, NodeUiQ, NodeUiQReadOnlyItem};
use crate::term::prelude::*;

const INTERSECTION_CHAR: [char; 16] = [
    ' ', '?', '?', '└', '?', '│', '┌', '├', '?', '┘', '─', '┴', '┐', '┤', '┬', '┼',
];

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

pub fn border_style_for(
    // available_moves: &Option<HashSet<Point>>,
    // available_moves_type: usize, // TODO something nicer
    // node_cursor: &NodeCursor,
    node_data: &NodeUiQReadOnlyItem,

    draw_config: &DrawConfiguration,
    x_range: &RangeInclusive<usize>, // TODO usize -> u32?
    y_range: &RangeInclusive<usize>, // TODO include if this border space is empty
) -> UiFormat {
    let color_scheme = draw_config.color_scheme();

    let NodeCursor(UVec2 {
        x: cursor_x,
        y: cursor_y,
    }) = node_data.node_cursor;

    // TODO optimized logic so we don't create a full set of points for every square
    if x_range.contains(&(*cursor_x as usize)) && y_range.contains(&(*cursor_y as usize)) {
        color_scheme.selected_square_border()
    }
    // else if check attacks are selected and range
    // color_scheme.attack_action(),
    else if !node_data.available_moves.is_empty()
        && !points_in_range(x_range, y_range).is_disjoint(node_data.available_moves)
    {
        color_scheme.possible_movement()
    } else {
        color_scheme.grid_border_default()
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
