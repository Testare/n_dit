use bevy::app::AppExit;
use bevy::ecs::query::Has;
use game_core::node::{AccessPoint, CurrentTurn, InNode, Node, NodeOp, NodePiece, OnTeam};
use game_core::op::{CoreOps, OpResult};
use game_core::player::{ForPlayer, Player};

use super::node_popups::{HelpMenu, OptionsMenu};
use crate::input_event::{MouseButton, MouseEventTty, MouseEventTtyDisabled, MouseEventTtyKind};
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
pub struct QuitButton;

pub fn mouse_button_menu(
    mut res_core_ops: ResMut<CoreOps>,
    mut evr_mouse: EventReader<MouseEventTty>,
    ready_buttons: Query<&ForPlayer, With<ReadyButton>>,
    quit_buttons: Query<(), With<QuitButton>>,
    end_turn_button: Query<AsDerefCopied<ForPlayer>, With<EndTurnButton>>,
    options_button: Query<AsDerefCopied<ForPlayer>, With<OptionsButton>>,
    help_button: Query<AsDerefCopied<ForPlayer>, With<HelpButton>>,
    mut options_menu: IndexedQuery<
        ForPlayer,
        AsDerefMut<VisibilityTty>,
        (With<OptionsMenu>, Without<HelpMenu>),
    >,
    mut help_menu: IndexedQuery<
        ForPlayer,
        AsDerefMut<VisibilityTty>,
        (With<HelpMenu>, Without<OptionsMenu>),
    >,

    mut evw_app_exit: EventWriter<AppExit>,
) {
    for mouse_event in evr_mouse.read() {
        if !matches!(
            mouse_event.event_kind(),
            MouseEventTtyKind::Down(MouseButton::Left)
        ) {
            continue;
        }
        if let Ok(for_player) = ready_buttons.get(mouse_event.entity()) {
            res_core_ops.request(**for_player, NodeOp::ReadyToGo);
        } else if quit_buttons.contains(mouse_event.entity()) {
            evw_app_exit.send(AppExit);
        } else if let Ok(for_player) = end_turn_button.get(mouse_event.entity()) {
            res_core_ops.request(for_player, NodeOp::EndTurn);
        } else if let Ok(for_player) = options_button.get(mouse_event.entity()) {
            if let Ok(mut options_vis) = options_menu.get_for_mut(for_player) {
                *options_vis = !*options_vis;
            }
            if let Ok(mut help_vis) = help_menu.get_for_mut(for_player) {
                help_vis.set_if_neq(false);
            }
        } else if let Ok(for_player) = help_button.get(mouse_event.entity()) {
            if let Ok(mut help_vis) = help_menu.get_for_mut(for_player) {
                *help_vis = !*help_vis;
            }
            if let Ok(mut options_vis) = options_menu.get_for_mut(for_player) {
                options_vis.set_if_neq(false);
            }
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
