use std::ops::Deref;

use game_core::card::{Action, Actions};
use game_core::node::{AccessPoint, ActiveCurio, IsTapped, Node, NodePiece};
use game_core::player::Player;

use super::{AvailableActionTargets, PlayerUiQ, SelectedAction, SelectedEntity};
use crate::node_ui::{AvailableMoves, TelegraphedAction};
use crate::prelude::*;

pub fn get_range_of_action(
    ast_actions: Res<Assets<Action>>,
    mut players: ParamSet<(
        Query<PlayerUiQ>,
        Query<(Entity, &mut AvailableActionTargets)>,
    )>,
    changed_player: Query<
        (),
        (
            With<Player>,
            Or<(
                Changed<SelectedAction>,
                Changed<SelectedEntity>,
                Changed<AvailableMoves>,
                Changed<TelegraphedAction>,
            )>,
        ),
    >,
    changed_access_point: Query<(), Changed<AccessPoint>>,
    node_pieces: Query<(&Actions, Option<&IsTapped>), With<NodePiece>>,
    node_grids: Query<(&EntityGrid, AsDerefCopied<ActiveCurio>), With<Node>>,
) {
    let players_to_update: HashSet<Entity> = players
        .p0()
        .iter()
        .filter_map(|player_q| {
            if changed_player.contains(player_q.entity)
                || player_q.selected_entity.of(&changed_access_point).is_some()
            {
                Some(player_q.entity)
            } else {
                None
            }
        })
        .collect();

    let mut action_target_updates: HashMap<Entity, HashSet<UVec2>> = players
        .p0()
        .iter_many(&players_to_update)
        .filter_map(|player_q| {
            // Note: Will probably have to change this logic so that when the player is
            // actually trying to perform the action, it only shows up

            let (curio_actions, is_tapped) = player_q.selected_entity.of(&node_pieces)?;
            if is_tapped.map(|is_tapped| **is_tapped).unwrap_or(false) {
                return None;
            }
            let (grid, active_curio) = node_grids.get(**player_q.in_node).ok()?;

            let (action_id, entity) = match player_q.telegraphed_action.as_ref() {
                Some(action_id) => (action_id, active_curio?),
                None => (
                    &curio_actions[(**player_q.selected_action)?],
                    (**player_q.selected_entity)?,
                ),
            };
            let range = ast_actions.get(action_id)?.range()?; // Not all actions have a range
            let available_moves = player_q.available_moves.deref();
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
                        if range.in_range_of(grid, entity, pt) {
                            return Some(pt);
                        }

                        // TODO only run this if the player has selected to perform an action
                        if range.in_range_of_pts(available_moves, pt) {
                            return Some(pt);
                        }

                        None
                    })
                })
                .collect();
            Some((player_q.entity, pts))
        })
        .collect();
    for (player_id, mut available_action_targets) in players.p1().iter_mut() {
        if !players_to_update.contains(&player_id) {
            continue;
        }
        let new_available_actions = action_target_updates.remove(&player_id).unwrap_or_default();
        if new_available_actions != available_action_targets.0 {
            available_action_targets.0 = new_available_actions;
        }
    }
}
