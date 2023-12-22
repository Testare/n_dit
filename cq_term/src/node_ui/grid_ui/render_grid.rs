use std::cmp;

use charmi::{CharacterMapImage, CharmieRow};
use crossterm::style::ContentStyle;
use game_core::node::{ActiveCurio, Node};
use game_core::player::{ForPlayer, Player};
use game_core::registry::Reg;
use itertools::Itertools;

use super::borders::{arrow_border, border_style_for, intersection_for_pivot, BorderType};
use super::grid_animation::GridUiAnimation;
use super::render_square::render_square;
use super::{GridUi, NodePieceQ, PlayerUiQ, PlayerUiQItem, Scroll2d};
use crate::animation::AnimationPlayer;
use crate::base_ui::HoverPoint;
use crate::configuration::DrawConfiguration;
use crate::layout::CalculatedSizeTty;
use crate::node_ui::node_glyph::NodeGlyph;
use crate::prelude::*;
use crate::render::TerminalRendering;

const CLOSED_SQUARE: &str = "  ";
const OPEN_SQUARE: &str = "░░";

pub fn render_grid_system(
    node_data: Query<(&EntityGrid, &ActiveCurio), With<Node>>,
    node_pieces: Query<NodePieceQ>,
    players: Query<PlayerUiQ, With<Player>>,
    reg_glyph: Res<Reg<NodeGlyph>>,
    draw_config: Res<DrawConfiguration>,
    mut render_grid_q: Query<
        (
            &CalculatedSizeTty,
            &Scroll2d,
            &ForPlayer,
            &HoverPoint,
            &mut TerminalRendering,
        ),
        With<GridUi>,
    >,
    grid_animation: Query<
        (&AnimationPlayer, &TerminalRendering, &ForPlayer),
        (With<GridUiAnimation>, Without<GridUi>),
    >,
) {
    for (size, scroll, ForPlayer(player), hover_point, mut rendering) in render_grid_q.iter_mut() {
        if let Ok(player_ui_q) = players.get(*player) {
            if let Ok((grid, active_curio)) = node_data.get(**player_ui_q.in_node) {
                let grid_animation = grid_animation
                    .iter()
                    .find(|(_, _, for_player)| *player == ***for_player);
                if grid_animation.is_none() {
                    log::error!("Cannot find attack animation for player {:?}.", player);
                    continue;
                }
                let grid_rendering = render_grid(
                    size,
                    scroll,
                    hover_point,
                    &player_ui_q,
                    grid,
                    active_curio,
                    &node_pieces,
                    &reg_glyph,
                    &draw_config,
                    grid_animation.unwrap(),
                );

                rendering.update_charmie(grid_rendering);
            }
        }
    }
}

fn render_grid(
    size: &CalculatedSizeTty,
    scroll: &Scroll2d,
    hover_point: &HoverPoint,
    player_q: &PlayerUiQItem,
    grid: &EntityGrid,
    active_curio: &ActiveCurio,
    node_pieces: &Query<NodePieceQ>,
    reg_glyph: &Reg<NodeGlyph>,
    draw_config: &DrawConfiguration,
    grid_animation: (&AnimationPlayer, &TerminalRendering, &ForPlayer),
) -> CharacterMapImage {
    // TODO Break DrawConfiguration down into parts and resources

    let default_style = ContentStyle::new();

    let hover_point =
        hover_point.map(|UVec2 { x, y }| UVec2::new((x + scroll.x) / 3, (y + scroll.y) / 2)); // TODO make this a helper function?
    let width = grid.width() as usize;
    let height = grid.height() as usize;
    let grid_map = grid.number_map();

    let sprite_map = grid.point_map(|i, sprite| {
        render_square(i, sprite, active_curio, node_pieces, reg_glyph, draw_config)
    });

    let x_start = (scroll.x / 3) as usize;
    // The highest x value to be on screen, in character columns
    let x2 = cmp::min(width * 3 + 1, scroll.x as usize + size.width());
    let x_end = (x2 - 1) / 3;
    let skip_x = (scroll.x % 3) as usize; // Number of character columns to skip on first grid column
    let drop_x = (3 - (x2 % 3)) % 3;

    let y_start = (scroll.y / 2) as usize;
    let y_end = cmp::min(height, (scroll.y + size.height32() / 2) as usize);
    let skip_y = (scroll.y % 2) as usize;
    let keep_last_space = skip_y + size.height() % 2 == 0;
    let draw_arrow_border = active_curio.and_then(|active_curio_id| {
        let active_curio_q = node_pieces.get(active_curio_id).ok()?;
        if active_curio_q.speed.is_none()
            || active_curio_q.moves_taken.is_none()
            || active_curio_q.speed == active_curio_q.moves_taken
        {
            // If it has no speed, or it has already taken all of its moves
            return None;
        }

        let hover_point = hover_point?;
        if grid.square_is_closed(hover_point) {
            return None;
        }
        let curio_head = grid.head(active_curio_id)?;
        if hover_point.manhattan_distance(&curio_head) != 1 {
            return None;
        }
        if let Some(hover_entity) = grid.item_at(hover_point) {
            if hover_entity != active_curio_id {
                let entity_under_hover = node_pieces.get(hover_entity).ok()?;
                if entity_under_hover.has_curio {
                    // A curio that is not the active curio means you can't move there
                    return None;
                }
            }
        }
        let dir = curio_head.dirs_to(&hover_point)[0]
            .expect("we should always have one direction to a point 1 manhattan distance away");
        Some((
            match dir {
                Compass::North | Compass::West => curio_head,
                Compass::East | Compass::South => hover_point,
            },
            dir,
        ))
    });

    let (border_lines, mut space_lines): (Vec<CharmieRow>, Vec<CharmieRow>) = (y_start..=y_end)
        .map(|y| {
            let mut border_line = CharmieRow::new();
            let mut space_line = CharmieRow::new(); //String::with_capacity(str_width);
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

                let render_left_border = x != x_start || skip_x == 0;
                let render_half_space =
                    (x == x_start && skip_x == 2) || (x == x_end && drop_x == 1);
                let render_full_space = x != x_end || drop_x == 0; // && (x != x_start || skip_x != 2), but the "else" block handles that case

                if render_left_border {
                    if include_border {
                        let pivot_format = border_style_for(
                            player_q,
                            &hover_point,
                            draw_arrow_border.is_some(),
                            draw_config,
                            &border_x_range,
                            &border_y_range,
                        );
                        border_line.add_styled_text(pivot_format.apply(intersection_for_pivot(
                            &[left1, left2],
                            &[right1, right2],
                            draw_config,
                        )));
                    }
                    if include_space {
                        // Add first vertical border
                        let border_style = border_style_for(
                            player_q,
                            &hover_point,
                            draw_arrow_border.is_some(),
                            draw_config,
                            &border_x_range,
                            &(y..=y),
                        );

                        if let Some((_, dir)) = draw_arrow_border.filter(|(pt, dir)| {
                            x == pt.x as usize && y == pt.y as usize && dir.is_horizontal()
                        }) {
                            space_line.add_text(arrow_border(dir, false), &border_style);
                        } else {
                            space_line.add_styled_text(
                                border_style.apply(
                                    BorderType::of(left2, right2).vertical_border(draw_config),
                                ),
                            );
                        };
                    }
                }
                if render_half_space {
                    if include_border {
                        let border_style = border_style_for(
                            player_q,
                            &hover_point,
                            draw_arrow_border.is_some(),
                            draw_config,
                            &(x..=x),
                            &border_y_range,
                        );

                        if let Some((_, dir)) = draw_arrow_border.filter(|(pt, dir)| {
                            x == pt.x as usize && y == pt.y as usize && dir.is_vertical()
                        }) {
                            border_line.add_text(arrow_border(dir, true), &border_style);
                        } else {
                            border_line.add_styled_text(
                                border_style.apply(
                                    BorderType::of(right1, right2)
                                        .horizontal_border(draw_config)
                                        .chars()
                                        .next()
                                        .unwrap(),
                                ),
                            );
                        };
                    }
                    if include_space {
                        let space_style = space_style_for(x, y, player_q, draw_config);
                        let (square_style, square) = sprite_map
                            .get(&pt)
                            .map(|(style, square)| (style, square.as_ref()))
                            .unwrap_or_else(|| {
                                if grid.square_is_closed(pt) {
                                    (&default_style, CLOSED_SQUARE)
                                } else {
                                    (&default_style, OPEN_SQUARE)
                                }
                            });
                        let combined_style = charmi::add_content_styles(&space_style, square_style);
                        if square.chars().count() == 1 {
                            space_line.add_char(draw_config.half_char(), &combined_style);
                        } else {
                            // Whether we are getting the left half or the right half
                            let char_index = if x == x_start { 1 } else { 0 };
                            let half_char = square
                                .chars()
                                .nth(char_index)
                                .expect("there should be at least 2 characters");

                            space_line.add_char(half_char, &combined_style);
                        }
                    }
                } else if render_full_space {
                    if include_border {
                        let border_style = border_style_for(
                            player_q,
                            &hover_point,
                            draw_arrow_border.is_some(),
                            draw_config,
                            &(x..=x),
                            &border_y_range,
                        );

                        if let Some((_, dir)) = draw_arrow_border.filter(|(pt, dir)| {
                            x == pt.x as usize && y == pt.y as usize && dir.is_vertical()
                        }) {
                            border_line.add_text(arrow_border(dir, false), &border_style);
                        } else {
                            border_line.add_styled_text(border_style.apply(
                                BorderType::of(right1, right2).horizontal_border(draw_config),
                            ));
                        }
                    }
                    if include_space {
                        let space_style = space_style_for(x, y, player_q, draw_config);
                        let (square_style, square) = sprite_map
                            .get(&pt)
                            .map(|(style, square)| (style, square.as_str()))
                            .unwrap_or_else(|| {
                                if grid.square_is_closed(pt) {
                                    (&default_style, CLOSED_SQUARE)
                                } else {
                                    (&default_style, OPEN_SQUARE)
                                }
                            });

                        let combined_style = charmi::add_content_styles(&space_style, square_style);
                        space_line.add_text(square, &combined_style);
                    }
                }
            }
            (border_line, space_line)
        })
        .unzip();
    space_lines.truncate(height); // Still used for when the height isn't specified
    let charmi: CharacterMapImage = Itertools::interleave(border_lines.into_iter(), space_lines)
        .skip(skip_y)
        .take(size.height())
        .collect();

    if grid_animation.0.is_playing() {
        let clipped_attack = grid_animation.1.charmie().clip(
            scroll.x,
            scroll.y,
            size.width32(),
            size.height32(),
            Default::default(),
        );
        charmi.draw(&clipped_attack, 0, 0, Default::default())
    } else {
        charmi
    }
}

fn space_style_for(
    x: usize,
    y: usize,
    player_q: &PlayerUiQItem,
    draw_config: &DrawConfiguration,
) -> ContentStyle {
    if !player_q.cursor_is_hidden
        && x as u32 == player_q.node_cursor.x
        && y as u32 == player_q.node_cursor.y
    {
        draw_config.color_scheme().selected_square()
    } else {
        ContentStyle::default()
    }
}
