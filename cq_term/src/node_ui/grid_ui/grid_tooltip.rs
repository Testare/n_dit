use std::borrow::Cow;

use game_core::card::Action;
use game_core::node::{
    ActiveCurio, CurrentTurn, InNode, Node, NodePiece, OnTeam, Pickup, Team, TeamPhase,
};
use game_core::player::{ForPlayer, Player};

use super::{GridUi, NodePieceQ, NodePieceQItem};
use crate::base_ui::{HoverPoint, Scroll2d, Tooltip};
use crate::node_ui::{AvailableActionTargets, AvailableMoves, SelectedAction, SelectedEntity};
use crate::prelude::*;

const EMPTY_TOOLTIP: &str = "";

pub fn sys_grid_ui_tooltip(
    ast_actions: Res<Assets<Action>>,
    mut grid_uis: Query<
        (
            AsDerefMut<Tooltip>,
            AsDerefCopied<ForPlayer>,
            AsDerefCopied<HoverPoint>,
            AsDeref<Scroll2d>,
        ),
        With<GridUi>,
    >,
    players: Query<
        (
            AsDerefCopied<InNode>,
            AsDerefCopied<OnTeam>,
            AsDerefCopied<SelectedEntity>,
            AsDeref<AvailableMoves>,
            AsDeref<AvailableActionTargets>,
            AsDerefCopied<SelectedAction>,
        ),
        With<Player>,
    >,
    nodes: Query<
        (
            AsDerefCopied<ActiveCurio>,
            AsDerefCopied<CurrentTurn>,
            &EntityGrid,
        ),
        With<Node>,
    >,
    node_pieces: Query<NodePieceQ>,
    pickups: Query<(), (With<Pickup>, With<NodePiece>)>,
    teamcheck: Query<AsDerefCopied<OnTeam>>,
    teams: Query<&TeamPhase, With<Team>>,
) {
    for (mut tooltip, player_id, hover_point, scroll) in grid_uis.iter_mut() {
        let tooltip_text: Option<Cow<'static, str>> = hover_point.and_then(|hover_point| {
            let hover_point = UVec2::new(
                (scroll.x + hover_point.x) / 3,
                (scroll.y + hover_point.y) / 2,
            );

            let (
                node_id,
                player_team,
                selected_entity,
                available_moves,
                available_action_targets,
                selected_action,
            ) = players.get(player_id).ok()?;
            let (active_curio_id, current_turn, grid) = nodes.get(node_id).ok()?;
            let hover_entity = grid.item_at(hover_point);
            let hover_piece =
                hover_entity.and_then(|hover_entity| node_pieces.get(hover_entity).ok());
            let hover_name = hover_piece.as_ref().map(node_piece_q_name);
            let hover_team =
                hover_entity.and_then(|hover_entity| node_pieces.get(hover_entity).ok()?.team);
            let hover_piece_is_tapped = hover_piece.and_then(|hp| hp.is_tapped).unwrap_or(true);

            let team_phase = *teams.get(current_turn).ok()?;

            let left_click = hover_name
                .as_ref()
                .map(|hover_name| format!("[LeftMb] Look at {}", hover_name));
            let right_click = if active_curio_id.is_some() && active_curio_id == selected_entity {
                let active_curio_id =
                    active_curio_id.expect("We should have just checked that active_curio is SOME");
                let node_piece_q = node_pieces.get(active_curio_id).ok()?; // TODO debug assert here
                let curio_head = grid.head(active_curio_id)?;
                if available_moves.contains(&hover_point)
                    && curio_head.manhattan_distance(&hover_point) == 1
                {
                    let name = node_piece_q_name(&node_piece_q);
                    let dir = curio_head.dirs_to(&hover_point)[0]
                        .expect("manhattan distance of 1 means there should be at least one dir");
                    if let Some(pickup_id) =
                        hover_entity.filter(|hover_entity| pickups.contains(*hover_entity))
                    {
                        let pickup_name = node_piece_q_name(
                            &node_pieces
                                .get(pickup_id)
                                .expect("Should be a node piece if it is in the grid"),
                        );

                        Some(format!(
                            "[RightMb] Move {name} one space {dir} and pick up {pickup_name}"
                        ))
                    } else {
                        Some(format!("[RightMb] Move {name} one space {dir}"))
                    }
                } else if available_moves.is_empty()
                    && selected_action.is_some()
                    && available_action_targets.contains(&hover_point)
                {
                    let selected_action =
                        selected_action.expect("Selected action should be checked as some");
                    let action_handle = &node_piece_q
                        .actions
                        .expect("if selected action is some, actions should be some")
                        [selected_action];
                    let action = ast_actions
                        .get(action_handle)
                        .expect("action should be loaded");
                    let valid = action.target().valid_target(
                        grid,
                        active_curio_id,
                        hover_point,
                        |node_piece_id| teamcheck.get(node_piece_id).ok(),
                    );
                    let action_name = action.id();
                    let name = hover_name.unwrap_or("space".to_string());
                    if valid {
                        Some(format!("[RightMb] use {action_name} on {name}"))
                    } else {
                        Some(format!("[Cannot apply {action_name} to {name}]"))
                    }
                } else {
                    None
                }
            } else if player_team == current_turn
                && Some(player_team) == hover_team
                && team_phase == TeamPhase::Play
                && !hover_piece_is_tapped
            {
                let name = hover_name.unwrap();
                Some(format!("[RightMb] Activate {name}"))
            } else {
                None
            };
            match (left_click, right_click) {
                (Some(left_click), Some(right_click)) => {
                    Some(Cow::from(format!("{left_click} {right_click}")))
                },
                (Some(tooltip), None) | (None, Some(tooltip)) => Some(Cow::from(tooltip)),
                (None, None) => None,
            }

            // If active curio is a thing and selected entity is active curio
            //    If hoverpoint is one away from curio head and available_movement contains hoverpoint
            //      If contains pickup: [Right click] move <CURIO> <DIR> and pickup item
            //    If hoverpoint is is within action range
            //      If target is valid: Rightclick to apply <ACTION> to (<CURIO> or <COORD>)
            //      If target is invalid: (Invalid target for <ACTION>)
            // If hoverpoint is on a node piece:
            //    If it is yours:
            //      If it is a curio, and it is your turn, it isn't tapped, and it isn't active: [Left click] Look at <NODE PIECE> [Right click] Activate <CURIO>
            //    Otherwise: [Left click] Look at <NODE PIECE>

            // If active curio is a thing and selected entity is active curio
            //    If hoverpoint is one away from curio head and available_movement contains hoverpoint
            //      If empty: [Right click] move <CURIO> <DIR>
            //      If contains pickup: [Left click] Look at <NODE PIECE> [Right click] move <CURIO> <DIR> and pickup item
            //    If hoverpoint is is within action range
            //      If target is valid: Rightclick to apply <ACTION> to (<CURIO> or <COORD>)
            //      If target is invalid: Invalid target for <ACTION>
            // If hoverpoint is on a node piece:
            //    If it is yours:
            //      If is an access point: [Left click] Look at access point
            //      If it is a curio, and it is your turn, it isn't tapped, and it isn't active: [Left click] Look at <NODE PIECE> [Right click] Activate <CURIO>
            //    Otherwise: [Left click] Look at <NODE PIECE>
        });
        tooltip.set_if_neq(tooltip_text.unwrap_or(Cow::from(EMPTY_TOOLTIP)));
    }
}

fn node_piece_q_name(node_piece_q: &NodePieceQItem) -> String {
    node_piece_q
        .curio
        .map(|curio| curio.name())
        .or_else(|| node_piece_q.access_point.map(|_| "Access Point"))
        .or_else(|| {
            node_piece_q.pickup.map(|p| match p {
                Pickup::Card(_) => "card",
                Pickup::Item(_) => "item",
                Pickup::MacGuffin => "documents", // TODO string replacement
                Pickup::Mon(_) => "money",
            })
        })
        .unwrap_or("???")
        .to_string()
}
