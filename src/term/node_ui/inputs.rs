use crossterm::event::KeyEvent;
use game_core::card::Actions;
use game_core::node::{ActiveCurio, CurrentTurn, InNode, Node, NodeOp, NodePiece, OnTeam};
use game_core::player::{ForPlayer, Player};

use super::grid_ui::GridUi;
use super::menu_ui::MenuUiActions;
use super::{MessageBarUi, NodeCursor, SelectedAction, SelectedEntity};
use crate::term::key_map::NamedInput;
use crate::term::layout::{
    ui_focus_cycle_next, ui_focus_cycle_prev, StyleTty, UiFocus, UiFocusCycleOrder, UiFocusNext,
};
use crate::term::prelude::*;
use crate::term::{KeyMap, Submap};

pub fn advance_message_ui(
    mut ev_keys: EventReader<KeyEvent>,
    mut message_bar_ui: Query<(&mut MessageBarUi, &ForPlayer)>,
    players: Query<(Entity, &KeyMap), With<Player>>,
) {
    for KeyEvent { code, modifiers } in ev_keys.iter() {
        for (player, key_map) in players.iter() {
            key_map
                .named_input_for_key(Submap::Node, *code, *modifiers)
                .and_then(|named_input| {
                    if matches!(named_input, NamedInput::NextMsg) {
                        for (mut msg_bar, ForPlayer(for_player)) in message_bar_ui.iter_mut() {
                            if *for_player == player {
                                if msg_bar.len() > 0 {
                                    msg_bar.0 = msg_bar.0[1..].into();
                                }
                                break;
                            }
                        }
                    }
                    Some(())
                });
        }
    }
}

pub fn grid_ui_keyboard_controls(
    nodes: Query<(&EntityGrid, &ActiveCurio, &CurrentTurn), With<Node>>,
    mut players: Query<
        (
            Entity,
            &InNode,
            &OnTeam,
            &UiFocus,
            &KeyMap,
            &mut NodeCursor,
            &mut SelectedEntity,
            &mut SelectedAction,
        ),
        With<Player>,
    >,
    mut ev_keys: EventReader<KeyEvent>,
    mut ev_node_op: EventWriter<Op<NodeOp>>,
    grid_uis: Query<(), With<GridUi>>,
) {
    for KeyEvent { code, modifiers } in ev_keys.iter() {
        for (
            player,
            InNode(node),
            OnTeam(team),
            UiFocus(focus_opt),
            key_map,
            mut cursor,
            selected_entity,
            selected_action,
        ) in players.iter_mut()
        {
            if focus_opt
                .map(|focused_ui| !grid_uis.contains(focused_ui))
                .unwrap_or(false)
            {
                // If there is a focus and it isn't grid_ui, don't do grid_ui controls
                continue;
            }

            key_map
                .named_input_for_key(Submap::Node, *code, *modifiers)
                .and_then(|named_input| {
                    let (grid, active_curio, turn) = get_assert!(*node, nodes)?;
                    let is_controlling_active_curio = active_curio.is_some() && **turn == *team;

                    match named_input {
                        NamedInput::Direction(dir) => {
                            if is_controlling_active_curio {
                                ev_node_op.send(Op::new(player, NodeOp::MoveActiveCurio { dir }));
                            } else {
                                let next_cursor_pt = (**cursor + dir).min(grid.index_bounds());

                                cursor.adjust_to(
                                    next_cursor_pt,
                                    selected_entity,
                                    selected_action,
                                    grid,
                                )
                            }
                        },
                        NamedInput::Ready => {
                            ev_node_op.send(Op::new(player, NodeOp::ReadyToGo));
                        },
                        NamedInput::Activate => {
                            if is_controlling_active_curio {
                                ev_node_op.send(Op::new(player, NodeOp::DeactivateCurio));
                            } else if let Some(curio_id) = **selected_entity {
                                ev_node_op
                                    .send(Op::new(player, NodeOp::ActivateCurio { curio_id }));
                            }
                        },
                        _ => {},
                    }
                    Some(())
                });
        }
    }
}

pub fn action_menu_ui_controls(
    mut players: Query<
        (
            Entity,
            &mut UiFocus,
            &KeyMap,
            &SelectedEntity,
            &mut SelectedAction,
        ),
        With<Player>,
    >,
    node_pieces: Query<(&Actions,), With<NodePiece>>,
    mut ev_keys: EventReader<KeyEvent>,
    action_menu_uis: Query<(), With<MenuUiActions>>,
    grid_uis: Query<(Entity, &ForPlayer), With<GridUi>>,
) {
    for KeyEvent { code, modifiers } in ev_keys.iter() {
        for (player, mut focus, key_map, selected_entity, mut selected_action) in players.iter_mut()
        {
            if (**focus)
                .map(|focused_ui| !action_menu_uis.contains(focused_ui))
                .unwrap_or(true)
            {
                continue;
            }

            key_map
                .named_input_for_key(Submap::Node, *code, *modifiers)
                .and_then(|named_input| {
                    if let Some((actions,)) = selected_entity.of(&node_pieces) {
                        match named_input {
                            NamedInput::Direction(dir) => {
                                let actions_bound = actions.len();
                                let current_action = selected_action.unwrap_or(0);
                                let next_action = Some(
                                    (current_action
                                        + match dir {
                                            Compass::North => actions_bound - 1,
                                            Compass::South => 1,
                                            _ => 0,
                                        })
                                        % actions_bound,
                                );
                                if **selected_action != next_action {
                                    **selected_action = next_action;
                                }
                            },
                            NamedInput::Activate => {
                                if let Some((grid_ui_id, _)) = grid_uis
                                    .iter()
                                    .find(|(_, ForPlayer(ui_player))| *ui_player == player)
                                {
                                    **focus = Some(grid_ui_id);
                                }
                            },
                            NamedInput::MenuFocusNext | NamedInput::MenuFocusPrev => {
                                if let Some((grid_ui_id, _)) = grid_uis
                                    .iter()
                                    .find(|(_, ForPlayer(ui_player))| *ui_player == player)
                                {
                                    **selected_action = None;
                                    **focus = Some(grid_ui_id);
                                }
                            },
                            _ => {},
                        }
                    }
                    Some(())
                });
        }
    }
}

pub fn action_menu_on_focus(
    mut players: Query<(&UiFocus, &mut SelectedAction), (Changed<UiFocus>, With<Player>)>,
    action_menus: Query<(Entity, &ForPlayer), With<MenuUiActions>>,
) {
    for (action_menu, ForPlayer(player)) in action_menus.iter() {
        if let Ok((ui_focus, mut selected_action)) = players.get_mut(*player) {
            if **ui_focus == Some(action_menu) && selected_action.is_none() {
                **selected_action = Some(0);
            }
        }
    }
}

pub fn ui_focus_cycle(
    mut players: Query<(Entity, &mut UiFocusNext, &KeyMap), With<Player>>,
    mut ev_keys: EventReader<KeyEvent>,
    ui_nodes: Query<(Entity, &StyleTty, &UiFocusCycleOrder, &ForPlayer)>,
) {
    for KeyEvent { code, modifiers } in ev_keys.iter() {
        for (player, mut ui_focus, key_map) in players.iter_mut() {
            key_map
                .named_input_for_key(Submap::Node, *code, *modifiers)
                .and_then(|named_input| {
                    match named_input {
                        NamedInput::MenuFocusNext => {
                            let next_ui_focus =
                                ui_focus_cycle_next(**ui_focus, player, 0, &ui_nodes);
                            **ui_focus = next_ui_focus;
                        },
                        NamedInput::MenuFocusPrev => {
                            let next_ui_focus =
                                ui_focus_cycle_prev(**ui_focus, player, 0, &ui_nodes);
                            **ui_focus = next_ui_focus;
                        },
                        _ => {},
                    }
                    Some(())
                });
        }
    }
}
