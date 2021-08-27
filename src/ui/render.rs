use super::super::game::{Node, Piece};
use super::{DrawConfiguration, DrawType, FillMethod, Window};
use itertools::Itertools;
use std::cmp;

const INTERSECTION_CHAR: [char; 16] = [
    ' ', '?', '?', '└', '?', '│', '┌', '├', '?', '┘', '─', '┴', '┐', '┤', '┬', '┼',
];

#[derive(PartialEq, Eq)]
enum BorderType {
    Borderless = 0,
    Bordered = 1,
    Linked = 2,
}

impl Piece {
    // Might want to change this to just accept a mutable Write reference to make more effecient.
    fn render_square(&self, position: usize, configuration: &DrawConfiguration) -> String {
        match self {
            Piece::AccessPoint => String::from("&&"),
            Piece::Mon(_) => String::from("$$"),
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
        }
    }
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

impl Node {
    pub fn draw_node(
        &self,
        window: Option<Window>,
        draw_config: &DrawConfiguration,
    ) -> Vec<String> {
        // Math! Done here to optimize loops
        let grid = self.grid();
        let width = grid.width();
        let height = grid.height();
        let grid_map = grid.number_map();
        let bounds = window.unwrap_or_default();

        let piece_map = grid.point_map(|i, piece| piece.render_square(i, draw_config));

        let str_width = width * 3 + 3;
        let x_start = bounds.scroll_x / 3;
        let x2 = cmp::min(width * 3 + 1, bounds.scroll_x + bounds.width.get());
        let x_end = (x2 - 1) / 3;
        let skip_x = bounds.scroll_x % 3;
        let y_start = bounds.scroll_y / 2;
        let y_end = cmp::min(height, (bounds.scroll_y + bounds.height.get() - 1) / 2);
        let skip_y = bounds.scroll_y % 2;
        let keep_last_space = skip_y + bounds.height.get() % 2 == 0;

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

                    if include_border {
                        border_line.push(intersection_for_pivot(
                            &[left1, left2],
                            &[right1, right2],
                            draw_config,
                        ));
                    }

                    if include_space {
                        space_line.push(BorderType::of(left2, right2).vertical_border(draw_config));
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
                                    border_line.push(
                                        BorderType::of(right1, right2)
                                            .horizontal_border(draw_config)
                                            .chars()
                                            .next()
                                            .unwrap(),
                                    );
                                }
                                if include_space {
                                    let square =
                                        piece_map.get(&(x, y)).map(String::as_ref).unwrap_or("  ");
                                    if square.chars().count() == 1 {
                                        space_line.push(draw_config.half_char());
                                    } else {
                                        space_line.push(square.chars().next().unwrap());
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
                        border_line.push_str(
                            BorderType::of(right1, right2).horizontal_border(draw_config),
                        );
                    }
                    if include_space {
                        let square = piece_map.get(&(x, y)).map(String::as_ref).unwrap_or("  ");
                        space_line.push_str(square);
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
            .take(bounds.height.get())
            .collect()
    }
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
