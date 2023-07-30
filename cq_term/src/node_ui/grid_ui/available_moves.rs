use game_core::card::MovementSpeed;
use game_core::node::{AccessPoint, InNode, IsTapped, MovesTaken, Node, NodeOp, NodePiece, Pickup};
use game_core::player::Player;

use super::super::{AvailableMoves, NodeCursor, SelectedEntity};
use super::GridUi;
use crate::layout::UiFocus;
use crate::node_ui::SelectedAction;
use crate::prelude::*;

pub fn sys_adjust_available_moves(
    mut players: Query<
        (
            Entity,
            Ref<UiFocus>,
            &SelectedAction,
            &SelectedEntity,
            &InNode,
            &mut AvailableMoves,
        ),
        (With<Player>,),
    >,
    mut ev_node_op: EventReader<Op<NodeOp>>,
    changed_cursor: Query<(), Changed<NodeCursor>>,
    node_grids: Query<(&EntityGrid,), With<Node>>,
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
    grid_uis: Query<(), With<GridUi>>,
) {
    let players_who_performed_op: HashSet<Entity> =
        ev_node_op.iter().map(|op| op.player()).collect();
    for (player, ui_focus, selected_action, selected_entity, node_id, mut available_moves) in
        players.iter_mut()
    {
        if !changed_cursor.contains(player)
            && !players_who_performed_op.contains(&player)
            && !ui_focus.is_changed()
        {
            continue;
        }
        let new_moves = node_grids
            .get(**node_id)
            .ok()
            .and_then(|(grid,)| {
                let (entity, speed, moves_taken, tapped) = selected_entity.of(&node_pieces)?;
                if matches!(tapped, Some(IsTapped(true))) {
                    return None;
                }
                let head = grid
                    .head(entity)
                    .expect("a selected entity should exist in the grid map");

                if selected_action.is_some()
                    && ui_focus
                        .into_inner()
                        .map(|focused_entity| grid_uis.contains(focused_entity))
                        .unwrap_or(true)
                {
                    return Some(std::iter::once(head).collect());
                }
                let moves =
                    (**speed).saturating_sub(moves_taken.map(|mt| **mt).unwrap_or_default());
                let mut points_set = HashSet::new();

                possible_moves(head, &mut points_set, &pickups, moves, entity, &grid);
                Some(points_set)
            })
            .unwrap_or_default();

        if **available_moves != new_moves {
            **available_moves = new_moves;
            log::debug!("Available moves updated: {:?}", available_moves);
        }
    }
}

fn possible_moves(
    head: UVec2,
    points_set: &mut HashSet<UVec2>,
    pickup_query: &Query<(), With<Pickup>>,
    moves: u32,
    id: Entity,
    grid: &EntityGrid,
) {
    let mut last_edge_set: HashSet<_> = [head].into_iter().collect();
    for _ in 0..moves {
        let mut next_edge_set = HashSet::new();
        for pt in last_edge_set {
            for dir in Compass::ALL_DIRECTIONS.iter() {
                let next_pt = (pt + *dir).min(grid.index_bounds());
                if points_set.contains(&next_pt) {
                    continue;
                }
                let can_move_to_pt = grid.square_is_free(next_pt)
                    || grid
                        .item_at(next_pt)
                        .map(|pt_id| id == pt_id || pickup_query.contains(pt_id))
                        .unwrap_or(false);
                if can_move_to_pt {
                    points_set.insert(next_pt);
                    next_edge_set.insert(next_pt);
                }
            }
        }
        last_edge_set = next_edge_set;
    }
}
