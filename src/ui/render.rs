use std::{cmp, collections::HashSet, ops::RangeInclusive};

use game_core::{Node, Pickup, Point, Sprite, Team};
use itertools::Itertools;
use pad::PadStr;

use super::{DrawConfiguration, DrawType, FillMethod, SuperState, UiFormat, Window};

const CLOSED_SQUARE: &str = "  ";
const OPEN_SQUARE: &str = "░░";

const INTERSECTION_CHAR: [char; 16] = [
    ' ', '?', '?', '└', '?', '│', '┌', '├', '?', '┘', '─', '┴', '┐', '┤', '┬', '┼',
];

#[derive(PartialEq, Eq)]
enum BorderType {
    Borderless = 0,
    Bordered = 1,
    Linked = 2,
}

// Might want to change this to just accept a mutable Write reference to make more effecient.
fn render_square(
    sprite: &Sprite,
    node: &Node,
    key: usize,
    position: usize,
    configuration: &DrawConfiguration,
) -> String {
    let string = match sprite {
        Sprite::AccessPoint(_) => String::from("&&"),
        Sprite::Pickup(pickup) => configuration
            .color_scheme()
            .mon()
            .apply(pickup.square_display()),
        Sprite::Curio(curio) => {
            if position == 0 {
                String::from(curio.display())
            } else if false {
                // Logic to format the last square differently if position + 1 == max_size and the
                // curio is selected, so that you can tell that moving will not grow the curio.
                String::from("[]")
            } else {
                match configuration.tail_appearance() {
                    FillMethod::NoFill => String::from("  "),
                    FillMethod::Brackets => String::from("[]"),
                    FillMethod::DotFill => String::from(".."),
                    FillMethod::HeadCopy => String::from(curio.display()),
                    FillMethod::Sequence => {
                        format!("{:02}", position)
                    }
                }
            }
        }
    };
    style(sprite, node, key, position, configuration).apply(string)
}

fn style(
    sprite: &Sprite,
    node: &Node,
    key: usize,
    position: usize,
    draw_config: &DrawConfiguration,
) -> UiFormat {
    match sprite {
        Sprite::Pickup(_) => draw_config.color_scheme().mon(),
        Sprite::AccessPoint(_) => draw_config.color_scheme().access_point(),
        Sprite::Curio(curio) => match curio.team() {
            Team::PlayerTeam => {
                if node.active_curio_key() == Some(key) {
                    draw_config.color_scheme().player_team_active()
                } else if curio.tapped() && position == 0 {
                    draw_config.color_scheme().player_team_tapped()
                } else {
                    draw_config.color_scheme().player_team()
                }
            }
            Team::EnemyTeam => draw_config.color_scheme().enemy_team(),
        },
    }
}

pub fn render_menu(state: &SuperState, height: usize, width: usize) -> Vec<String> {
    // TODO height checking + scrolling + etc
    // TODO render_menu when there is no node
    let node = state
        .game_state()
        .node()
        .expect("TODO what if there is no node?");
    let sprite_opt = node
        .active_curio_key()
        .and_then(|key| node.sprite(key))
        .or_else(|| {
            let pt: Point = state.selected_square();
            node.sprite_at(pt)
        });

    let mut base_vec = vec![String::from(""); height];
    let color_scheme = state.draw_config().color_scheme();
    if let Some(sprite) = sprite_opt {
        match sprite {
            Sprite::Pickup(Pickup::Mon(mon_val)) => {
                base_vec[2].push_str("Money");
                base_vec[3] = "=".repeat(width);
                base_vec[4].push('$');
                base_vec[4].push_str(mon_val.to_string().as_str());
            }
            Sprite::Pickup(Pickup::Item(item)) => {
                base_vec[2].push_str("Loose Item");
                base_vec[3] = "=".repeat(width);
                base_vec[4].push_str(item.name());
            }
            Sprite::Pickup(Pickup::Card(card)) => {
                base_vec[2].push_str("Loose Card");
                base_vec[3] = "=".repeat(width);
                base_vec[4].push_str(card);
            }
            Sprite::AccessPoint(card_id) => {
                base_vec[2].push_str("Access Pnt");
                base_vec[3] = "=".repeat(width);
                base_vec[4].push_str(format!("{:?}", card_id).as_str());
            }
            Sprite::Curio(curio) => {
                base_vec[1].push_str("Curio");
                base_vec[2] = "=".repeat(width);
                base_vec[3].push('[');
                base_vec[3].push_str(curio.display());
                base_vec[3].push(']');
                base_vec[4].push_str(curio.name());
                base_vec[5]
                    .push_str(format!("S:{} MS:{}", curio.speed(), curio.max_size()).as_str());
                base_vec[6] = "=".repeat(width);
                let selected_action = state.selected_action_index();
                for (i, action) in curio.actions().iter().enumerate() {
                    let mut action = action.with_exact_width(width);
                    if Some(i) == selected_action {
                        action = color_scheme.selected_menu_item().apply(action);
                    }
                    base_vec[7 + i].push_str(action.as_str());
                }
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
    for x in x_range.clone() {
        for y in y_range.clone() {
            set.insert((x, y));
        }
    }
    set
}

pub fn border_style_for(
    available_moves: &Option<HashSet<Point>>,
    available_moves_type: usize, // TODO something nicer
    state: &SuperState,
    x_range: &RangeInclusive<usize>,
    y_range: &RangeInclusive<usize>, // TODO include if this border space is empty
) -> UiFormat {
    let color_scheme = state.draw_config().color_scheme();
    let selected_square = state.selected_square();
    let (selected_x, selected_y) = selected_square;
    if x_range.contains(&selected_x) && y_range.contains(&selected_y) {
        color_scheme.selected_square_border()
        // TODO optimized logic so we don't create a full set of points for every square
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

    let sprite_map =
        grid.point_map(|key, i, sprite| render_square(sprite, node, key, i, draw_config));

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

    /*let mut available_moves = selected_sprite
    .map(|sprite_key| node.possible_moves(sprite_key))
    .unwrap_or(HashSet::default());*/
    let mut action_type = 1;

    let mut available_moves: Option<HashSet<Point>> = node.with_active_curio(|curio| {
        state
            .selected_action_index()
            .and_then(|action_index| curio.range_of_action(action_index))
            .map(|point_set| point_set.into_set())
    });

    if available_moves.is_none() {
        action_type = 0;
        available_moves = node.with_curio_at(state.selected_square(), |curio| {
            curio.possible_moves().into_set()
        });
    }

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
                        action_type,
                        state,
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
                    let border_style = border_style_for(
                        &available_moves,
                        action_type,
                        state,
                        &border_x_range,
                        &(y..=y),
                    );
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
                                    action_type,
                                    state,
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
                                let space_style = space_style_for(state, (x, y));
                                let square = sprite_map
                                    .get(&(x, y))
                                    .map(String::as_ref)
                                    .unwrap_or_else(|| {
                                        if grid.square_is_closed((x, y)) {
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
                        &available_moves,
                        action_type,
                        state,
                        &(x..=x),
                        &border_y_range,
                    );
                    border_line.push_str(
                        border_style
                            .apply(BorderType::of(right1, right2).horizontal_border(draw_config))
                            .as_str(),
                    );
                }
                if include_space {
                    let space_style = space_style_for(state, (x, y));
                    let square = sprite_map
                        .get(&(x, y))
                        .map(String::as_ref)
                        .unwrap_or_else(|| {
                            if grid.square_is_closed((x, y)) {
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
    let north = border_type_bit(draw_config, left[0], right[0]);
    let east = border_type_bit(draw_config, right[0], right[1]) << 1;
    let south = border_type_bit(draw_config, left[1], right[1]) << 2;
    let west = border_type_bit(draw_config, left[0], left[1]) << 3;

    INTERSECTION_CHAR[north | east | south | west]
}
