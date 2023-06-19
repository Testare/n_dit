use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
use game_core::node::{ActiveCurio, CurrentTurn, InNode, Node, NodeOp, OnTeam};
use game_core::player::{ForPlayer, Player};

use super::grid_ui::{GridUi, Scroll2D};
use super::{MessageBarUi, NodeCursor, SelectedAction, SelectedEntity};
use crate::term::layout::{CalculatedSizeTty, GlobalTranslationTty};
use crate::term::prelude::*;

pub fn grid_ui_keyboard_controls(
    node_grids: Query<(&EntityGrid, &ActiveCurio, &CurrentTurn), With<Node>>,
    mut players: Query<
        (
            &InNode,
            &OnTeam,
            &mut NodeCursor,
            &mut SelectedEntity,
            &mut SelectedAction,
        ),
        With<Player>,
    >,
    mut message_bar_ui: Query<&mut MessageBarUi>,
    grid_ui_view: Query<
        (
            &CalculatedSizeTty,
            &GlobalTranslationTty,
            &Scroll2D,
            &ForPlayer,
        ),
        With<GridUi>,
    >,
    mut inputs: EventReader<CrosstermEvent>,
    mut ev_node_op: EventWriter<Op<NodeOp>>,
) {
    for (size, translation, scroll, ForPlayer(player)) in grid_ui_view.iter() {
        if let Ok((
            InNode(node),
            OnTeam(team),
            mut cursor,
            mut selected_entity,
            mut selected_action,
        )) = players.get_mut(*player)
        {
            let (grid, active_curio, turn) = node_grids
                .get(*node)
                .expect("if a player is in a node, it should have an entity grid");

            let is_controlling_active_curio = active_curio.is_some() && **turn == *team;

            for input in inputs.iter() {
                match input {
                    CrosstermEvent::Key(KeyEvent {
                        code: KeyCode::Char(input_char),
                        ..
                    }) => match input_char {
                        'k' | 'w' => {
                            if is_controlling_active_curio {
                                ev_node_op.send(Op::new(
                                    *player,
                                    NodeOp::MoveActiveCurio {
                                        dir: Compass::North,
                                    },
                                ))
                            }
                            cursor.y = cursor.y.saturating_sub(1);
                        },
                        'h' | 'a' => {
                            if is_controlling_active_curio {
                                ev_node_op.send(Op::new(
                                    *player,
                                    NodeOp::MoveActiveCurio { dir: Compass::West },
                                ))
                            }
                            cursor.x = cursor.x.saturating_sub(1);
                        },
                        'j' | 's' => {
                            if is_controlling_active_curio {
                                ev_node_op.send(Op::new(
                                    *player,
                                    NodeOp::MoveActiveCurio {
                                        dir: Compass::South,
                                    },
                                ))
                            }
                            cursor.y = cursor.y.saturating_add(1).min(grid.height() - 1 as u32);
                        },
                        'l' | 'd' => {
                            if is_controlling_active_curio {
                                ev_node_op.send(Op::new(
                                    *player,
                                    NodeOp::MoveActiveCurio { dir: Compass::East },
                                ))
                            }
                            cursor.x = cursor.x.saturating_add(1).min(grid.width() - 1 as u32);
                        },
                        '-' => {
                            ev_node_op.send(Op::new(*player, NodeOp::ReadyToGo));
                        },
                        ' ' => {
                            if is_controlling_active_curio {
                                ev_node_op.send(Op::new(*player, NodeOp::DeactivateCurio));
                            } else if let Some(curio_id) = **selected_entity {
                                ev_node_op
                                    .send(Op::new(*player, NodeOp::ActivateCurio { curio_id }));
                            }
                        },
                        _ => {},
                    },
                    CrosstermEvent::Key(KeyEvent {
                        code: KeyCode::Enter,
                        ..
                    }) => {
                        // Next message
                        for mut msg_bar in message_bar_ui.iter_mut() {
                            msg_bar.0 = msg_bar.0[1..].into();
                        }
                    },
                    CrosstermEvent::Mouse(
                        event @ MouseEvent {
                            kind,
                            column,
                            row,
                            modifiers,
                        },
                    ) => {
                        // TODO Use layout events instead
                        let grid_contained = size.contains_mouse_event(translation, event);
                        if grid_contained {
                            match *kind {
                                MouseEventKind::Moved
                                    if modifiers.contains(KeyModifiers::SHIFT) =>
                                {
                                    let new_x = ((*column as u32) + scroll.x - translation.x) / 3;
                                    let new_y = ((*row as u32) + scroll.y - translation.y) / 2;
                                    if new_x < grid.width() && new_y < grid.height() {
                                        cursor.x = new_x;
                                        cursor.y = new_y;
                                    }
                                },
                                MouseEventKind::Down(MouseButton::Left) => {
                                    let new_x = ((*column as u32) + scroll.x - translation.x) / 3;
                                    let new_y = ((*row as u32) + scroll.y - translation.y) / 2;
                                    if new_x < grid.width() && new_y < grid.height() {
                                        if cursor.x == new_x && cursor.y == new_y {
                                            log::debug!("Click again on selected square")
                                        } else {
                                            cursor.x = new_x;
                                            cursor.y = new_y;
                                        }
                                    }
                                },
                                _ => {},
                            }
                        }
                    },
                    _ => {},
                }
            }
            if is_controlling_active_curio {
            } else {
                let now_selected_entity = grid.item_at(**cursor);
                if selected_entity.0 != now_selected_entity {
                    selected_entity.0 = now_selected_entity;
                    **selected_action = None;
                }
            }
        }
    }
}
