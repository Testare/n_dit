use bevy::app::AppExit;
use bevy::ecs::query::Has;
use crossterm::event::{MouseButton, MouseEventKind};
use game_core::node::{
    AccessPoint, AccessPointLoadingRule, CurrentTurn, InNode, IsReadyToGo, Node, NodeOp, NodePiece,
    OnTeam, Teams,
};
use game_core::op::OpResult;
use game_core::player::{ForPlayer, Player};

use crate::layout::{LayoutEvent, LayoutMouseTargetDisabled, VisibilityTty};
use crate::prelude::*;

#[derive(Clone, Copy, Component, Reflect)]
pub struct PauseButton;

#[derive(Clone, Copy, Component, Reflect)]
pub struct ReadyButton;

#[derive(Clone, Copy, Component, Reflect)]
pub struct EndTurnButton;

#[derive(Clone, Copy, Component, Reflect)]
pub struct HelpButton;

#[derive(Clone, Copy, Component, Reflect)]
pub struct QuitButton;

pub fn mouse_button_menu(
    mut evr_mouse: EventReader<LayoutEvent>,
    ready_buttons: Query<&ForPlayer, With<ReadyButton>>,
    quit_buttons: Query<(), With<QuitButton>>,
    end_turn_button: Query<AsDerefCopied<ForPlayer>, With<EndTurnButton>>,
    mut evw_node_op: EventWriter<Op<NodeOp>>,
    mut evw_app_exit: EventWriter<AppExit>,
) {
    for mouse_event in evr_mouse.iter() {
        if !matches!(
            mouse_event.event_kind(),
            MouseEventKind::Down(MouseButton::Left)
        ) {
            continue;
        }
        if let Ok(for_player) = ready_buttons.get(mouse_event.entity()) {
            NodeOp::ReadyToGo.for_p(**for_player).send(&mut evw_node_op);
        } else if quit_buttons.contains(mouse_event.entity()) {
            evw_app_exit.send(AppExit);
        } else if let Ok(for_player) = end_turn_button.get(mouse_event.entity()) {
            NodeOp::EndTurn.for_p(for_player).send(&mut evw_node_op);
        }
    }
}

pub fn sys_ready_button_disable(
    mut commands: Commands,
    mut ev_node_op_result: EventReader<OpResult<NodeOp>>,
    mut ready_buttons: IndexedQuery<
        ForPlayer,
        (
            Entity,
            Has<LayoutMouseTargetDisabled>,
            AsDerefMut<VisibilityTty>,
        ),
        (With<ReadyButton>, Without<EndTurnButton>),
    >,
    mut end_turn_buttons: IndexedQuery<
        ForPlayer,
        (
            Entity,
            Has<LayoutMouseTargetDisabled>,
            AsDerefMut<VisibilityTty>,
        ),
        (With<EndTurnButton>, Without<ReadyButton>),
    >,
    nodes: Query<(&EntityGrid, AsDerefCopied<CurrentTurn>), With<Node>>,
    players: Query<(Entity, AsDerefCopied<OnTeam>, AsDerefCopied<InNode>), With<Player>>,
    access_points: Query<(Entity, AsDerefCopied<OnTeam>, &AccessPoint), With<NodePiece>>,
) {
    // TODO make this a run condition?
    for node_op_result in ev_node_op_result.iter() {
        if let OpResult {
            result: Ok(_),
            source: Op { player, op },
        } = node_op_result
        {
            let updates = match op {
                NodeOp::EndTurn { .. } => {
                    log::debug!("NOCOMMIT A");
                    get_assert!(*player, players, |(_, _, node)| {
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
                    .unwrap_or_default()
                },
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
                        commands.entity(id).remove::<LayoutMouseTargetDisabled>();
                    } else if !button_is_disabled && !should_be_enabled {
                        commands.entity(id).insert(LayoutMouseTargetDisabled);
                    }
                }
                if let Ok((id, button_is_disabled, mut visibility)) =
                    end_turn_buttons.get_for_mut(player_id)
                {
                    if let Some(show_end_turn_button) = show_end_turn_button {
                        visibility.set_if_neq(show_end_turn_button);
                    }
                    if button_is_disabled && should_be_enabled {
                        commands.entity(id).remove::<LayoutMouseTargetDisabled>();
                    } else if !button_is_disabled && !should_be_enabled {
                        commands.entity(id).insert(LayoutMouseTargetDisabled);
                    }
                }
            }
        }
    }
}
