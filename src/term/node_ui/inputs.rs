use crossterm::event::KeyEvent;
use game_core::card::Actions;
use game_core::node::{ActiveCurio, CurrentTurn, InNode, Node, NodeOp, NodePiece, OnTeam};
use game_core::player::{Player, ForPlayer};

use super::grid_ui::GridUi;
use super::menu_ui::MenuUiActions;
use super::{MessageBarUi, NodeCursor, SelectedAction, SelectedEntity};
use crate::term::key_map::NamedInput;
use crate::term::layout::UiFocus;
use crate::term::prelude::*;
use crate::term::{KeyMap, Submap};

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
    mut message_bar_ui: Query<&mut MessageBarUi>,
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

            // if focus_opt.
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
                        NamedInput::NextMsg => {
                            // Next message
                            for mut msg_bar in message_bar_ui.iter_mut() {
                                msg_bar.0 = msg_bar.0[1..].into();
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
        for (player, mut focus, key_map, selected_entity, mut selected_action) in
            players.iter_mut()
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
                                let next_action = Some((current_action + match dir {
                                    Compass::North => actions_bound - 1,
                                    Compass::South => 1,
                                    _ => 0,
                                }) % actions_bound);
                                if **selected_action != next_action {
                                    **selected_action = next_action;
                                }
                            },
                            NamedInput::Activate => {
                                if let Some((grid_ui_id, _)) = grid_uis.iter().find(|(_, ForPlayer(ui_player))| *ui_player == player) {
                                    **focus = Some(grid_ui_id);
                                }
                            },
                            NamedInput::MenuFocusNext | NamedInput::MenuFocusPrev => {
                                if let Some((grid_ui_id, _)) = grid_uis.iter().find(|(_, ForPlayer(ui_player))| *ui_player == player) {
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
