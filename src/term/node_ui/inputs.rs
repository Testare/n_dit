use crossterm::event::KeyEvent;
use game_core::node::{ActiveCurio, CurrentTurn, InNode, Node, NodeOp, OnTeam};
use game_core::player::Player;

use super::{MessageBarUi, NodeCursor, SelectedAction, SelectedEntity};
use crate::term::key_map::NamedInput;
use crate::term::prelude::*;
use crate::term::{KeyMap, Submap};

pub fn grid_ui_keyboard_controls(
    nodes: Query<(&EntityGrid, &ActiveCurio, &CurrentTurn), With<Node>>,
    mut players: Query<
        (
            Entity,
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
    mut ev_keys: EventReader<KeyEvent>,
    mut ev_node_op: EventWriter<Op<NodeOp>>,
) {
    for KeyEvent { code, modifiers } in ev_keys.iter() {
        for (
            player,
            InNode(node),
            OnTeam(team),
            key_map,
            mut cursor,
            selected_entity,
            selected_action,
        ) in players.iter_mut()
        {
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
