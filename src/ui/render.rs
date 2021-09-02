use super::super::game::{Piece, Point, Team};
use super::{DrawConfiguration, DrawType, FillMethod, SuperState, UiFormat, Window};
use itertools::Itertools;
use std::{cmp, collections::HashSet, ops::RangeInclusive};

const INTERSECTION_CHAR: [char; 16] = [
    ' ', '?', '?', '└', '?', '│', '┌', '├', '?', '┘', '─', '┴', '┐', '┤', '┬', '┼',
];

#[derive(PartialEq, Eq)]
enum BorderType {
    Borderless = 0,
    Bordered = 1,
    Linked = 2,
}

// TODO render to arbitrary write object
impl Piece {
    // Might want to change this to just accept a mutable Write reference to make more effecient.
    // Might want to change this to accept SuperState to allow coloring sprites here.
    fn render_square(&self, state: &SuperState, position: usize, configuration: &DrawConfiguration) -> String {
        let string = match self {
            Piece::AccessPoint => String::from("&&"),
            Piece::Mon(_) => configuration.color_scheme().mon().apply("$$"),
            Piece::Program(sprite) => {
                if position == 0 {
                    String::from(sprite.display())
                } else if false {
                    // Logic to format the last square differently if position + 1 == max_size and the
                    // sprite is selected, so that you can tell that moving will not grow the sprite.
                    String::from("[]")
                } else {
                    match configuration.tail_appearance() {
                        FillMethod::NoFill => String::from("  "),
                        FillMethod::Brackets => String::from("[]"),
                        FillMethod::DotFill => String::from(".."),
                        FillMethod::HeadCopy => String::from(sprite.display()),
                        FillMethod::Sequence => {
                            format!("{:02}", position)
                        }
                    }
                }
            }
        };
        self.style(state, position, configuration).apply(string)
    }

    fn style(&self, state: &SuperState, _position: usize, draw_config: &DrawConfiguration) -> UiFormat {
        match self {
            Piece::Mon(_) => draw_config.color_scheme().mon(),
            Piece::AccessPoint => draw_config.color_scheme().access_point(),
            Piece::Program(sprite) => match sprite.team() {
                Team::PlayerTeam => {
                    if state.game.node().unwrap().active_sprite() == Some(sprite) {
                        draw_config.color_scheme().mon() // TODO active_sprite
                    } else {
                        draw_config.color_scheme().player_team()
                    }
                }
                Team::EnemyTeam => draw_config.color_scheme().enemy_team(),
            },
        }
    }
}

pub fn render_menu(state: &SuperState, height: usize, width: usize) -> Vec<String> {
    let pt: Point = state.selected_square();
    let piece_opt = state
        .game
        .node()
        .expect("TODO What if there is no node?")
        .piece_at(pt);
    let mut base_vec = vec![String::from(""); height];
    if let Some(piece) = piece_opt {
        match piece {
            Piece::Mon(mon_val) => {
                base_vec[2].push_str("Money");
                base_vec[3] = "=".repeat(width);
                base_vec[4].push('$');
                base_vec[4].push_str(mon_val.to_string().as_str());
            }
            Piece::AccessPoint => {
                base_vec[2].push_str("Access Pnt");
            }
            Piece::Program(sprite) => {
                base_vec[2].push_str("Program");
                base_vec[3] = "=".repeat(width);
                base_vec[4].push('[');
                base_vec[4].push_str(sprite.display());
                base_vec[4].push(']');
                base_vec[5].push_str(sprite.name());
            }
        };
    }
    base_vec
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

fn points_in_range(
    x_range: &RangeInclusive<usize>,
    y_range: &RangeInclusive<usize>,
) -> HashSet<Point> {
    let mut set = HashSet::default();
    for x in x_range.clone().into_iter() {
        for y in y_range.clone().into_iter() {
            set.insert((x, y));
        }
    }
    set
}

pub fn border_style_for(
    available_moves: &HashSet<Point>,
    state: &SuperState,
    x_range: &RangeInclusive<usize>,
    y_range: &RangeInclusive<usize>,
) -> UiFormat {
    let color_scheme = state.draw_config().color_scheme();
    let selected_square = state.selected_square();
    let (selected_x, selected_y) = selected_square;
    if x_range.contains(&selected_x) && y_range.contains(&selected_y) {
        color_scheme.selected_square_border()
        // TODO optimized logic so we don't create a full set of points for every square
    } else if !available_moves.is_disjoint(&points_in_range(x_range, y_range)) {
        color_scheme.possible_movement()
    } else {
        color_scheme.grid_border_default()
    }
}

pub fn space_style_for(state: &SuperState, pt: Point) -> UiFormat {
    if state.selected_square() == pt {
        state.draw_config().color_scheme().selected_square()
    } else {
        UiFormat::NONE
    }
}

// TODO make this unsafe, panic if there is no node
// TODO "NodeRenderingMathCache" struct
pub fn render_node(state: &SuperState, window: Window) -> Vec<String> {
    let node_opt = state.game_state().node();
    if node_opt.is_none() {
        return vec![]; // Panic?
    }
    let node = node_opt.unwrap();
    let draw_config = state.draw_config();
    let grid = node.grid();
    let width = grid.width();
    let height = grid.height();
    let grid_map = grid.number_map();

    let piece_map = grid.point_map(|i, piece| piece.render_square(&state, i, draw_config));

    let str_width = width * 3 + 3;
    let x_start = window.scroll_x / 3;
    let x2 = cmp::min(width * 3 + 1, window.scroll_x + window.width.get());
    let padding_size = window.width.get() + window.scroll_x - x2;
    let padding = " ".repeat(padding_size);

    let x_end = (x2 - 1) / 3;
    let skip_x = window.scroll_x % 3;
    let y_start = window.scroll_y / 2;
    let y_end = cmp::min(height, (window.scroll_y + window.height.get() - 1) / 2);
    let skip_y = window.scroll_y % 2;
    let keep_last_space = skip_y + window.height.get() % 2 == 0;

    let selected_piece = grid.item_key_at(state.selected_square());
    let available_moves = selected_piece
        .map(|piece_key| node.possible_moves(piece_key))
        .unwrap_or(HashSet::default());


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

                let border_x_range = if x == 0 { 0..=0 } else { x - 1..=x };

                let border_y_range = if y == 0 { 0..=0 } else { y - 1..=y };

                if include_border {
                    let pivot_format = border_style_for(
                        &available_moves,
                        &state,
                        &border_x_range,
                        &border_y_range,
                    );
                    border_line.push_str(
                        pivot_format
                            .apply(intersection_for_pivot(
                                &[left1, left2],
                                &[right1, right2],
                                draw_config,
                            ))
                            .as_str(),
                    );
                }

                if include_space {
                    let border_style =
                        border_style_for(&available_moves, &state, &border_x_range, &(y..=y));
                    space_line.push_str(
                        border_style
                            .apply(BorderType::of(left2, right2).vertical_border(draw_config))
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
                                    &available_moves,
                                    &state,
                                    &(x..=x),
                                    &border_y_range,
                                );
                                border_line.push_str(
                                    border_style
                                        .apply(
                                            BorderType::of(right1, right2)
                                                .horizontal_border(draw_config)
                                                .chars()
                                                .next()
                                                .unwrap(),
                                        )
                                        .as_str(),
                                );
                            }
                            if include_space {
                                let space_style = space_style_for(&state, (x, y));
                                let square =
                                    piece_map.get(&(x, y)).map(String::as_ref).unwrap_or("  ");
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
                    let border_style =
                        border_style_for(&available_moves, &state, &(x..=x), &border_y_range);
                    border_line.push_str(
                        border_style
                            .apply(BorderType::of(right1, right2).horizontal_border(draw_config))
                            .as_str(),
                    );
                }
                if include_space {
                    let space_style = space_style_for(&state, (x, y));
                    let square = piece_map.get(&(x, y)).map(String::as_ref).unwrap_or("  ");
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
        .take(window.height.get())
        .map(|mut row| {
            row.push_str(padding.as_str());
            row //.green().to_string()
        })
        .collect()
}

// Helper method for draw_node()
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
    let north = border_type_bit(&draw_config, left[0], right[0]);
    let east = border_type_bit(&draw_config, right[0], right[1]) << 1;
    let south = border_type_bit(&draw_config, left[1], right[1]) << 2;
    let west = border_type_bit(&draw_config, left[0], left[1]) << 3;

    INTERSECTION_CHAR[north | east | south | west]
}
