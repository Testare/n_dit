mod borders;
mod render_square;

use std::cmp;

use bevy::ecs::query::WorldQuery;
use game_core::card::MovementSpeed;
use game_core::node::{AccessPoint, IsTapped, MovesTaken, NodePiece, Pickup, Team};
use game_core::Direction;
use itertools::Itertools;

use self::borders::{border_style_for, intersection_for_pivot, BorderType};
use super::registry::GlyphRegistry;
use super::{AvailableMoves, NodeCursor, NodeUiDataParam, NodeUiQReadOnlyItem};
use crate::term::configuration::{DrawConfiguration, UiFormat};
use crate::term::layout::CalculatedSizeTty;
use crate::term::prelude::*;
use crate::term::render::UpdateRendering;

#[derive(Component)]
pub struct GridUi;

#[derive(Component, Debug, Default, Deref, DerefMut, FromReflect, Reflect)]
pub struct NodeViewScroll(pub UVec2);

const CLOSED_SQUARE: &str = "  ";
const OPEN_SQUARE: &str = "░░";

pub fn adjust_scroll(
    mut node_cursors: Query<(&NodeCursor, &EntityGrid)>,
    mut grid_ui_view: Query<(&CalculatedSizeTty, &mut NodeViewScroll), With<GridUi>>,
) {
    for (cursor, grid) in node_cursors.iter_mut() {
        if let Ok((size, mut scroll)) = grid_ui_view.get_single_mut() {
            scroll.x = scroll
                .x
                .min(cursor.x * 3) // Keeps node cursor from going off the left
                .max((cursor.x * 3 + 4).saturating_sub(size.width32())) // Keeps node cursor from going off the right
                .min((grid.width() * 3 + 1).saturating_sub(size.width32())); // On resize, show as much grid as possible
            scroll.y = scroll
                .y
                .min(cursor.y * 2) // Keeps node cursor from going off the right
                .min((grid.height() * 2 + 1).saturating_sub(size.height32())) // Keeps node cursor from going off the bottom
                .max((cursor.y * 2 + 3).saturating_sub(size.height32())); // On resize, show as much grid as possible
        }
    }
}

pub fn adjust_available_moves(
    mut moves_params: ParamSet<(
        Query<&mut AvailableMoves, Changed<NodeCursor>>,
        NodeUiDataParam,
    )>,
    pickup_query: Query<(), With<Pickup>>,
    node_pieces: Query<(&MovementSpeed, Option<&MovesTaken>, Option<&IsTapped>), With<NodePiece>>,
) {
    if moves_params.p0().is_empty() {
        return;
    }
    let node_ui_data = moves_params.p1();
    let new_moves = node_ui_data
        .node_data()
        .and_then(|node_data| {
            let entity = (**node_data.selected_entity)?;
            let (speed, moves_taken, tapped) = node_pieces.get(entity).ok()?;
            if matches!(tapped, Some(IsTapped(true))) {
                return None;
            }
            let moves = (**speed).saturating_sub(moves_taken.map(|mt| **mt).unwrap_or_default());
            let mut points_set = HashSet::new();
            let head = node_data
                .grid
                .head(entity)
                .expect("a selected entity should exist in the grid map with a head");

            possible_moves_recur(
                head,
                &mut points_set,
                &pickup_query,
                moves,
                node_data.grid.bounds(),
                entity,
                &node_data,
            );
            Some(points_set)
        })
        .unwrap_or_default();
    let mut available_moves_param = moves_params.p0();
    let mut available_moves = available_moves_param.single_mut();
    if **available_moves != new_moves {
        **available_moves = new_moves;
        log::debug!("Available moves updated: {:?}", available_moves);
    }
}

#[derive(WorldQuery)]
pub struct NodePieceQ {
    piece: &'static NodePiece,
    team: Option<&'static Team>,
    speed: Option<&'static MovementSpeed>,
    is_tapped: Option<&'static IsTapped>,
    access_point: Option<&'static AccessPoint>,
}

pub fn render_grid_system(
    mut commands: Commands,
    node_ui_data: NodeUiDataParam,
    node_pieces: Query<NodePieceQ>,
    glyph_registry: Res<GlyphRegistry>,
    draw_config: Res<DrawConfiguration>,
    render_grid_q: Query<(Entity, &CalculatedSizeTty, &NodeViewScroll), With<GridUi>>,
) {
    if let Some(node_data) = node_ui_data.node_data() {
        if let Ok((render_grid_id, size, scroll)) = render_grid_q.get_single() {
            let grid_rendering = render_grid(
                size,
                scroll,
                &node_data,
                &node_pieces,
                &glyph_registry,
                &draw_config,
            );

            commands
                .get_entity(render_grid_id)
                .unwrap()
                .update_rendering(grid_rendering);
        }
    }
}

fn render_grid(
    size: &CalculatedSizeTty,
    scroll: &NodeViewScroll,
    node_data: &NodeUiQReadOnlyItem,
    node_pieces: &Query<NodePieceQ>,
    glyph_registry: &GlyphRegistry,
    draw_config: &DrawConfiguration,
) -> Vec<String> {
    // TODO Break DrawConfiguration down into parts and resources

    let grid = node_data.grid;
    let node_cursor = node_data.node_cursor;

    let width = grid.width() as usize;
    let height = grid.height() as usize;
    let grid_map = grid.number_map();

    let sprite_map = grid.point_map(|i, sprite| {
        render_square::render_square(i, sprite, node_pieces, glyph_registry, &draw_config)
    });

    let str_width = width * 3 + 3;

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

                let render_left_border = x != x_start || skip_x == 0;
                let render_half_space =
                    (x == x_start && skip_x == 2) || (x == x_end && drop_x == 1);
                let render_full_space = x != x_end || drop_x == 0; // && (x != x_start || skip_x != 2), but the "else" block handles that case

                if render_left_border {
                    if include_border {
                        let pivot_format = border_style_for(
                            &node_data,
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
                        // Add first vertical border
                        let border_style = border_style_for(
                            &node_data,
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
                }
                if render_half_space {
                    if include_border {
                        let border_style = border_style_for(
                            &node_data,
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
                        let (square_style, square) = sprite_map
                            .get(&pt)
                            .map(|(style, square)| (style, square.as_ref()))
                            .unwrap_or_else(|| {
                                if grid.square_is_closed(pt) {
                                    (&UiFormat::NONE, CLOSED_SQUARE)
                                } else {
                                    (&UiFormat::NONE, OPEN_SQUARE)
                                }
                            });
                        if square.chars().count() == 1 {
                            space_line.push_str(
                                space_style
                                    .apply(square_style.apply(draw_config.half_char()))
                                    .as_str(),
                            );
                        } else {
                            // Whether we are getting the left half or the right half
                            let char_index = if x == x_start { 1 } else { 0 };
                            let half_char = square
                                .chars()
                                .nth(char_index)
                                .expect("there should be at least 2 characters");

                            space_line.push_str(
                                space_style.apply(square_style.apply(half_char)).as_str(),
                            );
                        }
                    }
                } else if render_full_space {
                    if include_border {
                        let border_style = border_style_for(
                            &node_data,
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
                                    BorderType::of(right1, right2).horizontal_border(&draw_config),
                                )
                                .as_str(),
                        );
                    }
                    if include_space {
                        let space_style = space_style_for(x, y, node_cursor, &draw_config);
                        let (square_style, square) = sprite_map
                            .get(&pt)
                            .map(|(style, square)| (style, square.as_str()))
                            .unwrap_or_else(|| {
                                if grid.square_is_closed(pt) {
                                    (&UiFormat::NONE, CLOSED_SQUARE)
                                } else {
                                    (&UiFormat::NONE, OPEN_SQUARE)
                                }
                            });
                        // TODO replace all calls to X.push_str(style.apply(y).as_str()) with style.push_str_to(&mut x (dest), y (addition))
                        // TODO Instead of applying two styles, compose the styles then apply
                        space_line.push_str(space_style.apply(square_style.apply(square)).as_str());
                    }
                }
            }
            (border_line, space_line)
        })
        .unzip();
    space_lines.truncate(height); // Still used for when the height isn't specified
    Itertools::interleave(border_lines.into_iter(), space_lines.into_iter())
        .skip(skip_y)
        .take(size.height())
        .collect()
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

fn possible_moves_recur(
    pt: UVec2,
    points_set: &mut HashSet<UVec2>,
    pickup_query: &Query<(), With<Pickup>>,
    moves: u32,
    bounds: UVec2,
    id: Entity,
    node_data: &NodeUiQReadOnlyItem,
) {
    if moves == 0 {
        return;
    }
    for dir in Direction::ALL_DIRECTIONS.iter() {
        let next_pt = (pt + *dir).min(bounds);
        if points_set.contains(&next_pt) {
            continue;
        }
        let can_move_to_pt = node_data.grid.square_is_free(next_pt)
            || node_data
                .grid
                .item_at(next_pt)
                .map(|pt_id| id == pt_id || pickup_query.contains(pt_id))
                .unwrap_or(false);
        // TODO If this is a pickup, it also works
        if can_move_to_pt {
            points_set.insert(next_pt);
            possible_moves_recur(
                next_pt,
                points_set,
                pickup_query,
                moves - 1,
                bounds,
                id,
                node_data,
            );
        }
    }
}
