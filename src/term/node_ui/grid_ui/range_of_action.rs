use std::ops::Deref;

use game_core::card::{Actions};
use game_core::node::{AccessPoint, IsTapped, Node, NodePiece};
use game_core::player::Player;

use super::{AvailableActionTargets, SelectedAction, SelectedEntity, PlayerUiQ};
use crate::term::prelude::*;

pub fn get_range_of_action(
    mut players: ParamSet<(
        Query<PlayerUiQ>,
        Query<(Entity, &mut AvailableActionTargets)>,
    )>,
    changed_player: Query<
        (),
        (
            With<Player>,
            Or<(Changed<SelectedAction>, Changed<SelectedEntity>)>,
        ),
    >,
    changed_access_point: Query<(), Changed<AccessPoint>>,
    node_pieces: Query<(&Actions, Option<&IsTapped>), With<NodePiece>>,
    node_grids: Query<&EntityGrid, With<Node>>,
) {
    let players_to_update: HashSet<Entity> = players
        .p0()
        .iter()
        .filter_map(|player_q| {
            if changed_player.contains(player_q.entity) {
                Some(player_q.entity)
            } else if player_q.selected_entity.of(&changed_access_point).is_some() {
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
        if !players_to_update.contains(&player_id) {
            continue;
        }
        let new_available_actions = action_target_updates.remove(&player_id).unwrap_or_default();
        if new_available_actions != available_action_targets.0 {
            available_action_targets.0 = new_available_actions;
        }
    }
}