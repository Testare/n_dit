use bevy::ecs::entity::EntityHashSet;
use bevy::ecs::query::Has;
use game_core::node::{
    AccessPoint, CurrentTurn, InNode, Node, NodeOp, NodePiece, NodeUndoStack, OnTeam, Team,
    TeamPhase,
};
use game_core::op::OpResult;
use game_core::player::{ForPlayer, Player};

use crate::input_event::MouseEventTtyDisabled;
use crate::layout::VisibilityTty;
use crate::prelude::*;

#[derive(Clone, Copy, Component, Reflect)]
pub struct OptionsButton;

#[derive(Clone, Copy, Component, Reflect)]
pub struct ReadyButton;

#[derive(Clone, Copy, Component, Reflect)]
pub struct EndTurnButton;

#[derive(Clone, Copy, Component, Reflect)]
pub struct HelpButton;

#[derive(Clone, Copy, Component, Reflect)]
pub struct UndoButton;

#[derive(Clone, Copy, Component, Reflect)]
pub struct QuitButton;

pub fn sys_ready_button_disable(
    mut commands: Commands,
    mut ev_node_op_result: EventReader<OpResult<NodeOp>>,
    mut ready_buttons: IndexedQuery<
        ForPlayer,
        (
            Entity,
            Has<MouseEventTtyDisabled>,
            AsDerefMut<VisibilityTty>,
        ),
        (With<ReadyButton>, Without<EndTurnButton>),
    >,
    mut end_turn_buttons: IndexedQuery<
        ForPlayer,
        (
            Entity,
            Has<MouseEventTtyDisabled>,
            AsDerefMut<VisibilityTty>,
        ),
        (With<EndTurnButton>, Without<ReadyButton>),
    >,
    nodes: Query<(&EntityGrid, AsDerefCopied<CurrentTurn>), With<Node>>,
    players: Query<(Entity, AsDerefCopied<OnTeam>, AsDerefCopied<InNode>), With<Player>>,
    access_points: Query<(Entity, AsDerefCopied<OnTeam>, &AccessPoint), With<NodePiece>>,
) {
    for node_op_result in ev_node_op_result.read() {
        if let OpResult {
            result: Ok(_),
            source: player,
            op,
        } = node_op_result
        {
            let updates = match op {
                NodeOp::EndTurn { .. } => get_assert!(*player, players, |(_, _, node)| {
                    let (_, current_turn) = get_assert!(node, nodes)?;
                    Some(
                        players
                            .iter()
                            .filter_map(|(player_id, player_team, player_node)| {
                                (player_node == node).then_some((
                                    player_id,
                                    None,
                                    current_turn == player_team,
                                ))
                            })
                            .collect(),
                    )
                })
                .unwrap_or_default(),
                NodeOp::LoadAccessPoint { .. } => vec![(*player, Some(false), true)],
                NodeOp::ReadyToGo => vec![(*player, Some(true), true)],
                NodeOp::UnloadAccessPoint { .. } => {
                    vec![get_assert!(*player, players, |(_, player_team, in_node)| {
                        let (grid, _) = get_assert!(in_node, nodes)?;
                        let still_can_go =
                            access_points.iter().any(|(id, ap_team, access_point)| {
                                grid.contains_key(id)
                                    && ap_team == player_team
                                    && access_point.card().is_some()
                            });
                        Some((*player, Some(false), still_can_go))
                    })
                    .unwrap_or((*player, None, false))]
                },
                _ => continue,
            };
            for (player_id, show_end_turn_button, should_be_enabled) in updates.into_iter() {
                if let Ok((id, button_is_disabled, mut visibility)) =
                    ready_buttons.get_for_mut(player_id)
                {
                    if let Some(show_end_turn_button) = show_end_turn_button {
                        visibility.set_if_neq(!show_end_turn_button);
                    }
                    if button_is_disabled && should_be_enabled {
                        commands.entity(id).remove::<MouseEventTtyDisabled>();
                    } else if !button_is_disabled && !should_be_enabled {
                        commands.entity(id).insert(MouseEventTtyDisabled);
                    }
                }
                if let Ok((id, button_is_disabled, mut visibility)) =
                    end_turn_buttons.get_for_mut(player_id)
                {
                    if let Some(show_end_turn_button) = show_end_turn_button {
                        visibility.set_if_neq(show_end_turn_button);
                    }
                    if button_is_disabled && should_be_enabled {
                        commands.entity(id).remove::<MouseEventTtyDisabled>();
                    } else if !button_is_disabled && !should_be_enabled {
                        commands.entity(id).insert(MouseEventTtyDisabled);
                    }
                }
            }
        }
    }
}

pub fn sys_undo_button_state(
    mut commands: Commands,
    mut evr_node_op: EventReader<OpResult<NodeOp>>,
    q_player: Query<AsDerefMut<OnTeam>, With<Player>>,
    q_team: Query<(&TeamPhase, &NodeUndoStack), With<Team>>,
    mut q_undo_button: Query<
        (
            Entity,
            &ForPlayer,
            AsDerefMut<VisibilityTty>,
            Has<MouseEventTtyDisabled>,
        ),
        With<UndoButton>,
    >,
) {
    let updated_teams: EntityHashSet = evr_node_op
        .read()
        .map(|op_result| op_result.source())
        .filter_map(|player_id| q_player.get(player_id).ok().copied())
        .collect();

    if updated_teams.is_empty() {
        return;
    }
    for (ui_id, &ForPlayer(player_id), mut is_visible, has_disable_component) in
        q_undo_button.iter_mut()
    {
        q_player.get(player_id).ok().and_then(|team_id| {
            if !updated_teams.contains(team_id) {
                return None;
            }
            let (team_phase, undo_stack) = q_team.get(*team_id).ok()?;
            let should_be_visible = *team_phase != TeamPhase::Setup;
            is_visible.set_if_neq(should_be_visible);
            // If the undo stack is empty, it should have disable component
            // If not visible, doesn't matter
            if should_be_visible && (undo_stack.is_empty() != has_disable_component) {
                if has_disable_component {
                    commands.entity(ui_id).remove::<MouseEventTtyDisabled>();
                } else {
                    commands.entity(ui_id).insert(MouseEventTtyDisabled);
                }
            }
            Some(())
        });
    }
}
