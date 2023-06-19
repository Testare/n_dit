use game_core::card::MovementSpeed;
use game_core::node::{AccessPoint, InNode, IsTapped, MovesTaken, Node, NodePiece, Pickup};
use game_core::player::Player;
use game_core::Compass;

use super::super::{AvailableMoves, NodeCursor, SelectedEntity};
use crate::term::prelude::*;

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
    for dir in Compass::ALL_DIRECTIONS.iter() {
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
