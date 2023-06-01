use super::registry::GlyphRegistry;
use super::{render_square, RenderNodeDataReadOnlyItem};
use crate::term::configuration::{DrawConfiguration, DrawType, UiFormat};
use crate::term::layout::CalculatedSizeTty;
use crate::term::node::NodeCursor;
use crate::term::TerminalWindow;
use bevy::prelude::*;
use game_core::{EntityGrid, NodePiece, Team};
use itertools::Itertools;
use std::cmp;
use std::ops::RangeInclusive;

const CLOSED_SQUARE: &str = "  ";
const OPEN_SQUARE: &str = "░░";
const ZWSP: char = '\u{200B}';
const EXAMPLE: char = '死';

const INTERSECTION_CHAR: [char; 16] = [
    ' ', '?', '?', '└', '?', '│', '┌', '├', '?', '┘', '─', '┴', '┐', '┤', '┬', '┼',
];

#[derive(PartialEq, Eq)]
enum BorderType {
    Borderless = 0,
    Bordered = 1,
    Linked = 2,
}

impl BorderType {
    fn of(lhs: usize, rhs: usize) -> BorderType {
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

    fn horizontal_border(&self, draw_config: &DrawConfiguration) -> &'static str {
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

    fn vertical_border(&self, draw_config: &DrawConfiguration) -> char {
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
    node_cursor: &NodeCursor,

    draw_config: &DrawConfiguration,
    x_range: &RangeInclusive<usize>,
    y_range: &RangeInclusive<usize>, // TODO include if this border space is empty
) -> UiFormat {
    let color_scheme = draw_config.color_scheme();

    let NodeCursor(UVec2 {
        x: cursor_x,
        y: cursor_y,
    }) = node_cursor;

    // TODO optimized logic so we don't create a full set of points for every square
    if x_range.contains(&(*cursor_x as usize)) && y_range.contains(&(*cursor_y as usize)) {
        color_scheme.selected_square_border()
    }
    /*
    } else if available_moves.is_some()
        && !available_moves
            .as_ref()
            .unwrap()
            .is_disjoint(&points_in_range(x_range, y_range))
    {
        match available_moves_type {
            0 => color_scheme.possible_movement(),
            _ => color_scheme.attack_action(),
        }
    */
    else {
        color_scheme.grid_border_default()
    }
}

pub fn render_grid(
    window: &Res<TerminalWindow>,
    size: &CalculatedSizeTty,
    node_data: &RenderNodeDataReadOnlyItem,
    node_pieces: &Query<(&NodePiece, Option<&Team>)>,
    glyph_registry: &GlyphRegistry,
) -> Vec<String> {
    // TODO Guardrail for when size is too small
    // TODO Break DrawConfiguration down into parts and resources

    let grid = node_data.grid;
    let node_cursor = node_data.node_cursor;

    let draw_config = DrawConfiguration::default();
    let width = grid.width() as usize;
    let height = grid.height() as usize;
    let grid_map = grid.number_map();

    let sprite_map = grid
        .point_map(|i, sprite| render_square(i, sprite, node_pieces, glyph_registry, &draw_config));

    let str_width = width * 3 + 3;
    let x_start = window.scroll_x() / 3;
    let x2 = cmp::min(width * 3 + 1, window.scroll_x() + size.width() as usize);

    let x_end = (x2 - 1) / 3;
    let skip_x = window.scroll_x() % 3;
    let y_start = window.scroll_y() / 2;
    let y_end = cmp::min(height, (window.scroll_y() + size.height() as usize - 1) / 2);
    let skip_y = window.scroll_y() % 2;
    let keep_last_space = skip_y + size.height() as usize % 2 == 0;

    /*
        let mut action_type = 1;

        let mut available_moves: Option<HashSet<Point>> = node.with_active_curio(|curio| {
            state
                .selected_action_index()
                .and_then(|action_index| action_index.checked_sub(1)) // Compensate for "No Action" option
                .and_then(|action_index| curio.range_of_action(action_index))
                .map(|point_set| point_set.into_set())
        });

        if available_moves.is_none() {
            action_type = 0;
            available_moves = node.with_curio_at(state.selected_square(), |curio| {
                curio.possible_moves().into_set()
            });
        }
    */
    // let mut available_moves = None;

    let (border_lines, mut space_lines): (Vec<String>, Vec<String>) = (y_start..=y_end)
        .map(|y| {
            let mut border_line = String::with_capacity(str_width);
            let mut space_line = String::with_capacity(str_width);
            let include_border = y != y_start || skip_y != 1;
            let include_space = y != height && (y != y_end || keep_last_space);
            for x in x_start..=x_end {
                let (left1, left2) = if x == 0 {
                    (0, 0)
                } else if y == 0 {
                    (0, grid_map[x - 1][0])
                } else if y == height {
                    (grid_map[x - 1][y - 1], 0)
                } else {
                    (grid_map[x - 1][y - 1], grid_map[x - 1][y])
                };

                let (right1, right2) = if x == width {
                    (0, 0)
                } else if y == 0 {
                    (0, grid_map[x][0])
                } else if y == height {
                    (grid_map[x][y - 1], 0)
                } else {
                    (grid_map[x][y - 1], grid_map[x][y])
                };
                let pt = (x as u32, y as u32).into();

                let border_x_range = if x == 0 { 0..=0 } else { x - 1..=x };

                let border_y_range = if y == 0 { 0..=0 } else { y - 1..=y };

                if include_border {
                    let pivot_format = border_style_for(
                        node_cursor,
                        &draw_config, // &available_moves,
                        // action_type,
                        // state,
                        &border_x_range,
                        &border_y_range,
                    );
                    border_line.push_str(
                        pivot_format
                            .apply(intersection_for_pivot(
                                &[left1, left2],
                                &[right1, right2],
                                &draw_config,
                            ))
                            .as_str(),
                    );
                }

                if include_space {
                    let border_style = border_style_for(
                        node_cursor,
                        &draw_config, /*
                                                              &available_moves,
                                                              action_type,
                                                              state,

                                      */
                        &border_x_range,
                        &(y..=y),
                    );
                    space_line.push_str(
                        border_style
                            .apply(BorderType::of(left2, right2).vertical_border(&draw_config))
                            .as_str(),
                    );
                }

                if x == x_end {
                    match x2 % 3 {
                        0 => {} // Continues on to the the normal operation
                        1 => {
                            break; // Already done
                        }
                        2 => {
                            // Only half the square is rendered
                            if include_border {
                                let border_style = border_style_for(
                                    node_cursor,
                                    &draw_config, /*
                                                  &available_moves,
                                                  action_type,
                                                  state,
                                                  */
                                    &(x..=x),
                                    &border_y_range,
                                );
                                border_line.push_str(
                                    border_style
                                        .apply(
                                            BorderType::of(right1, right2)
                                                .horizontal_border(&draw_config)
                                                .chars()
                                                .next()
                                                .unwrap(),
                                        )
                                        .as_str(),
                                );
                            }
                            if include_space {
                                let space_style = space_style_for(x, y, node_cursor, &draw_config);
                                let square =
                                    sprite_map.get(&pt).map(String::as_ref).unwrap_or_else(|| {
                                        if grid.square_is_closed(pt) {
                                            CLOSED_SQUARE
                                        } else {
                                            OPEN_SQUARE
                                        }
                                    });
                                if square.chars().count() == 1 {
                                    space_line.push_str(
                                        space_style.apply(draw_config.half_char()).as_str(),
                                    );
                                } else {
                                    space_line.push_str(
                                        space_style.apply(square.chars().next().unwrap()).as_str(),
                                    );
                                }
                            }
                            break;
                        }
                        _ => {
                            panic!("Impossible!")
                        }
                    }
                }
                if include_border {
                    let border_style = border_style_for(
                        node_cursor,
                        &draw_config, /*
                                                              &available_moves,
                                                              action_type,
                                                              state,
                                      */
                        &(x..=x),
                        &border_y_range,
                    );
                    border_line.push_str(
                        border_style
                            .apply(BorderType::of(right1, right2).horizontal_border(&draw_config))
                            .as_str(),
                    );
                }
                if include_space {
                    let space_style = space_style_for(x, y, node_cursor, &draw_config);
                    let square = sprite_map.get(&pt).map(String::as_ref).unwrap_or_else(|| {
                        if grid.square_is_closed(pt) {
                            CLOSED_SQUARE
                        } else {
                            OPEN_SQUARE
                        }
                    });
                    // TODO replace all calls to X.push_str(style.apply(y).as_str()) with style.push_str_to(&mut x (dest), y (addition))
                    space_line.push_str(space_style.apply(square).as_str());
                    if x == x_start && skip_x == 2 && square.chars().count() == 1 {
                        // To keep the grid aligned in the event of a double-width character.
                        space_line.push(draw_config.half_char());
                    }
                }
            }
            (
                border_line.chars().skip(skip_x).collect(),
                space_line.chars().skip(skip_x).collect(),
            )
        })
        .unzip();
    space_lines.truncate(height); // Still used for when the height isn't specified
    Itertools::interleave(border_lines.into_iter(), space_lines.into_iter())
        .skip(skip_y)
        .take(size.height() as usize)
        .collect()
}

fn intersection_for_pivot(
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

fn space_style_for(
    x: usize,
    y: usize,
    node_cursor: &NodeCursor,
    draw_config: &DrawConfiguration,
) -> UiFormat {
    if x as u32 == node_cursor.x && y as u32 == node_cursor.y {
        draw_config.color_scheme().selected_square()
    } else {
        UiFormat::NONE
    }
}
