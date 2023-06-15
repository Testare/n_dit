mod borders;
mod render_square;

use std::cmp;
use std::ops::Deref;

use bevy::ecs::query::WorldQuery;
use game_core::card::{Action, Actions, MovementSpeed};
use game_core::node::{AccessPoint, InNode, IsTapped, MovesTaken, Node, NodePiece, Pickup, Team};
use game_core::player::{ForPlayer, Player};
use game_core::Direction;
use itertools::Itertools;

use self::borders::{border_style_for, intersection_for_pivot, BorderType};
use super::registry::GlyphRegistry;
use super::{AvailableActionTargets, AvailableMoves, NodeCursor, SelectedAction, SelectedEntity};
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
    players: Query<(&NodeCursor, &InNode), With<Player>>,
    node_grids: Query<&EntityGrid, With<Node>>,
    mut ui: Query<(&CalculatedSizeTty, &mut NodeViewScroll, &ForPlayer), With<GridUi>>,
) {
    for (size, mut scroll, ForPlayer(player)) in ui.iter_mut() {
        if let Ok((cursor, InNode(node))) = players.get(*player) {
            if let Ok(grid) = node_grids.get(*node) {
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
}

pub fn adjust_available_moves(
    mut players: Query<(Entity, &SelectedEntity, &InNode, &mut AvailableMoves), (With<Player>,)>,
    changed_access_points: Query<(), Changed<AccessPoint>>,
    changed_cursor: Query<(), Changed<NodeCursor>>,
    node_grids: Query<&EntityGrid, With<Node>>,
    pickups: Query<(), With<Pickup>>,
    node_pieces: Query<
        (
            Entity,
            &MovementSpeed,
            Option<&MovesTaken>,
            Option<&IsTapped>,
        ),
        With<NodePiece>,
    >,
) {
    for (player, selected_entity, node_id, mut available_moves) in players.iter_mut() {
        if !changed_cursor.contains(player) {
            if selected_entity.of(&changed_access_points).is_none() {
                continue;
            }
        }
        let new_moves = node_grids
            .get(**node_id)
            .ok()
            .and_then(|grid| {
                let (entity, speed, moves_taken, tapped) = selected_entity.of(&node_pieces)?;
                if matches!(tapped, Some(IsTapped(true))) {
                    return None;
                }
                let moves =
                    (**speed).saturating_sub(moves_taken.map(|mt| **mt).unwrap_or_default());
                let mut points_set = HashSet::new();
                let head = grid
                    .head(entity)
                    .expect("a selected entity should exist in the grid map");

                possible_moves_recur(head, &mut points_set, &pickups, moves, entity, &grid);
                Some(points_set)
            })
            .unwrap_or_default();

        if **available_moves != new_moves {
            **available_moves = new_moves;
            log::debug!("Available moves updated: {:?}", available_moves);
        }
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

#[derive(WorldQuery)]
pub struct PlayerUiQ {
    entity: Entity,
    selected_entity: &'static SelectedEntity,
    selected_action: &'static SelectedAction,
    node_cursor: &'static NodeCursor,
    available_moves: &'static AvailableMoves,
    available_action_targets: &'static AvailableActionTargets,
    in_node: &'static InNode,
}

pub fn render_grid_system(
    mut commands: Commands,
    node_grids: Query<&EntityGrid, With<Node>>,
    node_pieces: Query<NodePieceQ>,
    players: Query<PlayerUiQ, With<Player>>,
    glyph_registry: Res<GlyphRegistry>,
    draw_config: Res<DrawConfiguration>,
    render_grid_q: Query<(Entity, &CalculatedSizeTty, &NodeViewScroll, &ForPlayer), With<GridUi>>,
) {
    for (render_grid_id, size, scroll, ForPlayer(player)) in render_grid_q.iter() {
        if let Ok(player_ui_q) = players.get(*player) {
            if let Ok(grid) = node_grids.get(**player_ui_q.in_node) {
                let grid_rendering = render_grid(
                    size,
                    scroll,
                    &player_ui_q,
                    &grid,
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
}

fn render_grid(
    size: &CalculatedSizeTty,
    scroll: &NodeViewScroll,
    player_q: &PlayerUiQItem,
    grid: &EntityGrid,
    node_pieces: &Query<NodePieceQ>,
    glyph_registry: &GlyphRegistry,
    draw_config: &DrawConfiguration,
) -> Vec<String> {
    // TODO Break DrawConfiguration down into parts and resources

    let node_cursor = player_q.node_cursor;

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
                            &player_q,
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
                            &player_q,
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
                            &player_q,
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
                            &player_q,
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
    id: Entity,
    grid: &EntityGrid,
) {
    if moves == 0 {
        return;
    }
    for dir in Direction::ALL_DIRECTIONS.iter() {
        let next_pt = (pt + *dir).min(grid.bounds());
        if points_set.contains(&next_pt) {
            continue;
        }
        let can_move_to_pt = grid.square_is_free(next_pt)
            || grid
                .item_at(next_pt)
                .map(|pt_id| id == pt_id || pickup_query.contains(pt_id))
                .unwrap_or(false);
        // TODO If this is a pickup, it also works
        if can_move_to_pt {
            points_set.insert(next_pt);
            possible_moves_recur(next_pt, points_set, pickup_query, moves - 1, id, grid);
        }
    }
}

pub fn get_range_of_action(
    mut players: ParamSet<(
        Query<
            PlayerUiQ,
            (
                With<Player>,
                Or<(Changed<SelectedAction>, Changed<SelectedEntity>)>,
            ),
        >,
        Query<
            (Entity, &mut AvailableActionTargets),
            (
                With<Player>,
                Or<(Changed<SelectedAction>, Changed<SelectedEntity>)>,
            ),
        >,
    )>,
    node_pieces: Query<(&Actions, Option<&IsTapped>), With<NodePiece>>,
    node_grids: Query<&EntityGrid, With<Node>>,
) {
    let mut action_target_updates: HashMap<Entity, HashSet<UVec2>> = players
        .p0()
        .iter()
        .filter_map(|player_q| {
            // Note: Will probably have to change this logic so that when the player is
            // actually trying to perform the action, it only shows up
            let (actions, is_tapped) = player_q.selected_entity.of(&node_pieces)?;
            if is_tapped.map(|is_tapped| **is_tapped).unwrap_or(false) {
                return None;
            }
            let action = &actions[(**player_q.selected_action)?];
            let available_moves = player_q.available_moves.deref();
            let entity = (**player_q.selected_entity)?;
            let grid = node_grids.get(**player_q.in_node).ok()?;
            let entity_head = grid.head(entity)?;
            let UVec2 {
                x: width,
                y: height,
            } = grid.bounds();
            let pts: HashSet<UVec2> = (0..width)
                .flat_map(|x| {
                    (0..height).filter_map(move |y| {
                        let pt = UVec2 { x, y };
                        // Will need to change this logic for Packman moves
                        if grid.square_is_closed(pt) {
                            return None;
                        }
                        // Will have to remove when I create actions that can target self
                        if grid.item_at(pt) == Some(entity) {
                            return None;
                        }
                        if available_moves.contains(&pt) {
                            return None;
                        }
                        if entity_head.x.abs_diff(pt.x) + entity_head.y.abs_diff(pt.y)
                            <= action.range
                        {
                            return Some(pt);
                        }
                        // TODO only run this if the player has selected to perform an action
                        for UVec2 { x, y } in available_moves.iter() {
                            // For some of the weird curio ideas I have, we'll need to make changes
                            // to this logic
                            if x.abs_diff(pt.x) + y.abs_diff(pt.y) <= action.range {
                                return Some(pt);
                            }
                        }
                        None
                    })
                })
                .collect();
            Some((player_q.entity, pts))
        })
        .collect();
    for (player_id, mut available_action_targets) in players.p1().iter_mut() {
        let new_available_actions = action_target_updates.remove(&player_id).unwrap_or_default();
        if new_available_actions != available_action_targets.0 {
            available_action_targets.0 = new_available_actions;
        }
    }
}
