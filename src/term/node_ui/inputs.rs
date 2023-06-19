use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
use game_core::node::{ActiveCurio, CurrentTurn, InNode, Node, NodeOp, OnTeam};
use game_core::player::{ForPlayer, Player};

use super::grid_ui::{GridUi, Scroll2D};
use super::{MessageBarUi, NodeCursor, SelectedAction, SelectedEntity};
use crate::term::layout::{CalculatedSizeTty, GlobalTranslationTty};
use crate::term::prelude::*;
use crate::term::{KeyMap, Submap};

pub fn grid_ui_keyboard_controls(
    node_grids: Query<(&EntityGrid, &ActiveCurio, &CurrentTurn), With<Node>>,
    mut players: Query<
        (
            &InNode,
            &OnTeam,
            &KeyMap,
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
    mut ev_keys: EventReader<KeyEvent>,
    mut ev_node_op: EventWriter<Op<NodeOp>>,
) {
    for (size, translation, scroll, ForPlayer(player)) in grid_ui_view.iter() {
        if let Ok((
            InNode(node),
            OnTeam(team),
            key_map,
            mut cursor,
            mut selected_entity,
            mut selected_action,
        )) = players.get_mut(*player)
        {
            let (grid, active_curio, turn) = node_grids
                .get(*node)
                .expect("if a player is in a node, it should have an entity grid");

            let is_controlling_active_curio = active_curio.is_some() && **turn == *team;

            for KeyEvent { code, modifiers } in ev_keys.iter() {
                if let Some(named_input) =
                    key_map.named_input_for_key(Submap::Node, *code, *modifiers)
                {
                    log::debug!("NAMED INPUT: {:?}", named_input);
                }
                match code {
                    KeyCode::Char(input_char) => match input_char {
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
                    KeyCode::Enter => {
                        // Next message
                        for mut msg_bar in message_bar_ui.iter_mut() {
                            msg_bar.0 = msg_bar.0[1..].into();
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
