use game_core::card::{Action, Actions, MovementSpeed};
use game_core::node::{
    AccessPoint, ActiveCurio, CurrentTurn, InNode, IsTapped, MovesTaken, Node, NodePiece, OnTeam,
    Pickup, Team, TeamPhase,
};
use game_core::player::{ForPlayer, Player};

use super::super::{AvailableMoves, SelectedNodePiece};
use super::{GridHoverPoint, GridUi, LastGridHoverPoint, PathToGridPoint, PlayerUiQ};
use crate::base_ui::{HoverPoint, Scroll2d};
use crate::layout::UiFocus;
use crate::node_ui::{AvailableActionTargets, CursorIsHidden, SelectedAction, TelegraphedAction};
use crate::prelude::*;

pub fn sys_adjust_available_moves(
    mut players: Query<
        (
            Ref<UiFocus>,
            &SelectedAction,
            AsDerefCopied<SelectedNodePiece>,
            &InNode,
            AsDerefCopiedOrDefault<CursorIsHidden>,
            &TelegraphedAction,
            &mut AvailableMoves,
        ),
        (With<Player>,),
    >,
    node_grids: Query<(&EntityGrid, AsDerefCopied<ActiveCurio>), With<Node>>,
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
    for (
        ui_focus,
        selected_action,
        selected_entity,
        node_id,
        cursor_is_hidden,
        telegraphed_action,
        mut available_moves,
    ) in players.iter_mut()
    {
        let new_moves = node_grids
            .get(**node_id)
            .ok()
            .and_then(|(grid, active_curio)| {
                if telegraphed_action.is_some() {
                    return None;
                }
                let curio_id = (!cursor_is_hidden)
                    .then_some(())
                    .and(selected_entity)
                    .or(active_curio)?;

                let (entity, speed, moves_taken, tapped) = node_pieces.get(curio_id).ok()?;
                if matches!(tapped, Some(IsTapped(true))) {
                    return None;
                }
                let head = grid.head(entity)?;

                if selected_action.is_some()
                    && ui_focus
                        .into_inner()
                        .map(|focused_entity| grid_uis.contains(focused_entity))
                        .unwrap_or(true)
                {
                    return Some(std::iter::once((head, None)).collect()); // TODO better pattern for this ?
                }
                let moves =
                    (**speed).saturating_sub(moves_taken.map(|mt| **mt).unwrap_or_default());
                Some(possible_moves(head, &pickups, moves, entity, grid))
            })
            .unwrap_or_default();

        if **available_moves != new_moves {
            **available_moves = new_moves;
        }
    }
}

fn possible_moves(
    head: UVec2,
    pickup_query: &Query<(), With<Pickup>>,
    moves: u32,
    id: Entity,
    grid: &EntityGrid,
) -> HashMap<UVec2, Option<Compass>> {
    let mut points_map: HashMap<UVec2, _> = [(head, None)].into_iter().collect();
    let mut last_edge_set: HashSet<_> = [head].into_iter().collect();
    for _ in 0..moves {
        let mut next_edge_set = HashSet::new();
        for pt in last_edge_set {
            for dir in Compass::ALL_DIRECTIONS.iter() {
                let next_pt = (pt + *dir).min(grid.index_bounds());
                if points_map.contains_key(&next_pt) {
                    continue;
                }
                // TODO only let certain entities get pickups.
                let can_move_to_pt = grid.square_is_free(next_pt)
                    || grid
                        .item_at(next_pt)
                        .map(|pt_id| id == pt_id || pickup_query.contains(pt_id))
                        .unwrap_or(false);
                if can_move_to_pt {
                    points_map.insert(next_pt, Some(*dir));
                    next_edge_set.insert(next_pt);
                }
            }
        }
        last_edge_set = next_edge_set;
    }
    points_map
}

// TODO might need to reorganize these methods
// Update Hover Grid Point.

pub fn sys_hover_grid_point(
    mut q_grid_ui: Query<
        (
            AsDerefCopied<HoverPoint>,
            AsDerefCopied<Scroll2d>,
            AsDerefMut<GridHoverPoint>,
            AsDerefMut<LastGridHoverPoint>,
        ),
        (With<GridUi>, Changed<HoverPoint>),
    >,
) {
    for (hover_point, scroll, mut grid_hover_point, mut last_grid_hover_point) in
        q_grid_ui.iter_mut()
    {
        let hover_point = hover_point.map(|pt| calculate_grid_pt(pt, scroll));
        grid_hover_point.set_if_neq(hover_point);
        if let Some(hover_point) = hover_point {
            last_grid_hover_point.set_if_neq(hover_point);
        }
    }
}

pub fn calculate_grid_pt(UVec2 { x, y }: UVec2, scroll: UVec2) -> UVec2 {
    UVec2::new((x + scroll.x) / 3, (y + scroll.y) / 2)
}

pub fn sys_path_under_hover(
    q_player: Query<
        (
            Entity,
            Ref<AvailableMoves>,
            AsDerefCopied<SelectedNodePiece>,
            &OnTeam,
            &InNode,
        ),
        With<Player>,
    >,
    mut grid_uis: Query<
        (
            AsDerefCopied<ForPlayer>,
            Ref<GridHoverPoint>,
            AsDerefMut<PathToGridPoint>,
        ),
        With<GridUi>,
    >,
    q_node: Query<&CurrentTurn, With<Node>>,
    q_node_piece: Query<&OnTeam, With<NodePiece>>,
    q_team: Query<&TeamPhase, With<Team>>,
) {
    for (player_id, available_moves, selected_entity, &OnTeam(player_team_id), &InNode(node_id)) in
        q_player.iter()
    {
        for (for_player, grid_hover_point, mut path_to_grid_point) in grid_uis.iter_mut() {
            if (!available_moves.is_changed() && !grid_hover_point.is_changed())
                || for_player != player_id
            {
                continue;
            }
            let selected_is_team_during_play = selected_entity
                .and_then(|selected_entity| {
                    let &OnTeam(selected_team_id) = q_node_piece.get(selected_entity).ok()?;
                    if selected_team_id != player_team_id {
                        return Some(false);
                    }
                    let &CurrentTurn(node_current_turn) = get_assert!(node_id, q_node)?;
                    if node_current_turn != selected_team_id {
                        return Some(false);
                    }
                    let team_phase = get_assert!(selected_team_id, q_team)?;
                    Some(*team_phase == TeamPhase::Play)
                })
                .unwrap_or(false);
            if !selected_is_team_during_play {
                path_to_grid_point.set_if_neq(Vec::default());
                continue;
            }
            let start =
                grid_hover_point.and_then(|pt| Some((pt, available_moves.get(&pt).copied()??)));
            let iter = std::iter::successors(start, |&(prev_pt, prev_dir)| {
                let next_pt = prev_pt - prev_dir;
                Some(next_pt).zip(available_moves.get(&next_pt).copied().flatten())
            });
            let mut path: Vec<_> = iter.collect();
            path.reverse();
            path_to_grid_point.set_if_neq(path);
        }
    }
}

pub fn sys_get_range_of_action(
    ast_actions: Res<Assets<Action>>,
    mut players: ParamSet<(
        Query<PlayerUiQ>,
        Query<(Entity, &mut AvailableActionTargets)>,
    )>,
    q_team: Query<AsDerefCopied<OnTeam>, With<NodePiece>>,
    changed_player: Query<
        (),
        (
            With<Player>,
            Or<(
                Changed<SelectedAction>,
                Changed<SelectedNodePiece>,
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

    let mut action_target_updates: HashMap<Entity, HashMap<UVec2, bool>> = players
        .p0()
        .iter_many(&players_to_update)
        .filter_map(|player_q| {
            // Note: Will probably have to change this logic so that when the player is
            // actually trying to perform the action, it only shows up
            let selected_piece = (*player_q.selected_entity.deref())?;

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
            let action_def = ast_actions.get(action_id)?;
            let target = action_def.target();
            let range = action_def.range()?; // Not all actions have a range
            let available_moves = player_q.available_moves.deref();
            let UVec2 {
                x: width,
                y: height,
            } = grid.bounds();

            let team_check = |id| q_team.get(id).ok();

            let pts: HashMap<UVec2, bool> = (0..width)
                .flat_map(|x| {
                    (0..height).filter_map(move |y| {
                        let pt = UVec2 { x, y };
                        let valid_target =
                            target.valid_target(grid, selected_piece, pt, team_check);
                        // Will need to change this logic for Packman moves
                        if !valid_target && grid.square_is_closed(pt) {
                            return None;
                        }
                        // Will have to remove when I create actions that can target self
                        if !valid_target && grid.item_at(pt) == Some(entity) {
                            return None;
                        }
                        // Maybe in the future we'll allow them to overlap so that you can attack things in range, but for now let's not
                        if available_moves.contains_key(&pt) {
                            return None;
                        }
                        if range.in_range_of(grid, entity, pt) {
                            return Some((pt, valid_target));
                        }

                        // TODO only run this if the player has selected to perform an action
                        if range.in_range_of_pts(available_moves.keys(), pt) {
                            return Some((pt, false));
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
