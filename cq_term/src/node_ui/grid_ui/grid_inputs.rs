use crossterm::event::KeyModifiers;
use game_core::card::{Action, Actions, NO_OP_ACTION_ID};
use game_core::node::{
    ActiveCurio, Curio, CurrentTurn, InNode, IsTapped, Node, NodeOp, NodePiece, OnTeam, Pickup,
    Team, TeamPhase,
};
use game_core::op::OpSubtype;
use game_core::opv2::{OpV2, PrimeOps};
use game_core::player::{ForPlayer, Player};

use super::{GridUi, Scroll2d};
use crate::input_event::{MouseButton, MouseEventTty, MouseEventTtyKind};
use crate::key_map::NamedInput;
use crate::layout::UiFocus;
use crate::node_ui::node_ui_op::FocusTarget;
use crate::node_ui::{NodeCursor, NodeUiOp, SelectedAction, SelectedEntity};
use crate::prelude::*;
use crate::{KeyMap, Submap};

pub fn handle_layout_events(
    ast_actions: Res<Assets<Action>>,
    mut ev_mouse: EventReader<MouseEventTty>,
    ui: Query<(&ForPlayer, &Scroll2d), With<GridUi>>,
    players: Query<
        (
            &InNode,
            &OnTeam,
            &NodeCursor,
            &SelectedEntity,
            &SelectedAction,
        ),
        With<Player>,
    >,
    nodes: Query<(&EntityGrid, &ActiveCurio, &CurrentTurn), With<Node>>,
    teams: Query<&TeamPhase, With<Team>>,
    pickups: Query<(), With<Pickup>>,
    curios: Query<(&OnTeam, &Actions, &IsTapped), With<Curio>>,
    mut ev_node_op: EventWriter<Op<NodeOp>>,
    mut ev_node_ui_op: EventWriter<Op<NodeUiOp>>,
) {
    for event in ev_mouse.read() {
        if let Ok((ForPlayer(player), scroll)) = ui.get(event.entity()) {
            if let MouseEventTtyKind::Down(button) = event.event_kind() {
                log::trace!("Clicked on the grid");
                NodeUiOp::ChangeFocus(FocusTarget::Grid)
                    .for_p(*player)
                    .send(&mut ev_node_ui_op);
                get_assert!(*player, players, |(
                    node,
                    team,
                    cursor,
                    selected_entity,
                    selected_action,
                )| {
                    let (grid, active_curio, current_turn) = get_assert!(**node, nodes)?;
                    let team_phase = get_assert!(**team, teams)?;
                    let clicked_pos = event.pos() + **scroll;
                    let clicked_node_pos = UVec2 {
                        x: clicked_pos.x / 3,
                        y: clicked_pos.y / 2,
                    };
                    let alternative_click = *button == MouseButton::Right
                        || (*button == MouseButton::Left
                            && (event
                                .modifiers()
                                .intersects(KeyModifiers::SHIFT | KeyModifiers::ALT)
                                || event.double_click()));
                    let is_controlling_active_curio =
                        active_curio.is_some() && **current_turn == **team;

                    if !alternative_click {
                        ev_node_ui_op
                            .send(NodeUiOp::MoveNodeCursor(clicked_node_pos.into()).for_p(*player))
                    } else {
                        let selected_action = selected_action.and_then(|selected_action| {
                            let (_, actions, tapped) = selected_entity.of(&curios)?;
                            if **tapped {
                                return None;
                            }
                            ast_actions.get(actions.get(selected_action)?)
                        });
                        let pt_in_range = selected_action
                            .as_ref()
                            .and_then(|selected_action| {
                                Some(selected_action.range()?.in_range_of(
                                    grid,
                                    selected_entity.unwrap(),
                                    clicked_node_pos,
                                ))
                            })
                            .unwrap_or(false);
                        if let (Some(action), true) = (selected_action, pt_in_range) {
                            ev_node_op.send(Op::new(
                                *player,
                                NodeOp::PerformCurioAction {
                                    action_id: action.id_cow(),
                                    curio: **selected_entity,
                                    target: clicked_node_pos,
                                },
                            ));
                        } else if is_controlling_active_curio {
                            let active_curio_id = active_curio.unwrap();
                            let head = grid.head(active_curio_id).unwrap();
                            // TODO A lot of this logic is duplicated in node_op. Finding a way
                            // to condense it would be great
                            if head.manhattan_distance(&clicked_node_pos) == 1 {
                                // TODO Possible better UI: Clicking on any moveable-square will move the curio one space in that direction
                                let valid_move_target = grid.square_is_free(clicked_node_pos)
                                    || if let Some(pt_key) = grid.item_at(clicked_node_pos) {
                                        pt_key == active_curio_id || pickups.contains(pt_key)
                                    } else {
                                        false
                                    };

                                if valid_move_target {
                                    if let [Some(dir), _] = head.dirs_to(&clicked_node_pos) {
                                        ev_node_op.send(Op::new(
                                            *player,
                                            NodeOp::MoveActiveCurio { dir },
                                        ));
                                        return Some(());
                                    }
                                }
                            }
                            // TODO try defaulting to some action

                            // If there was an action selected,
                            // If it was adjacent to the curio and open, move it there
                            // else if there was a default action and this is a valid target,
                            // apply it
                        } else if *team_phase == TeamPhase::Play {
                            if let Some(pt_key) = grid.item_at(**cursor) {
                                if let Ok((curio_team, _, _)) = curios.get(pt_key) {
                                    if curio_team == team {
                                        ev_node_op.send(Op::new(
                                            *player,
                                            NodeOp::ActivateCurio { curio_id: pt_key },
                                        ));
                                    }
                                }
                            }
                        }
                    }
                    Some(())
                });
            }
        }
    }
}

pub fn kb_grid(
    ast_actions: Res<Assets<Action>>,
    mut ev_keys: EventReader<KeyEvent>,
    mut res_prime_ops: ResMut<PrimeOps>,
    nodes: Query<(&EntityGrid, &ActiveCurio, &CurrentTurn), With<Node>>,
    players: Query<
        (
            Entity, // Solid candidate for making a WorldQuery derive
            &InNode,
            &OnTeam,
            &UiFocus,
            &KeyMap,
            &NodeCursor,
            &SelectedEntity,
            &SelectedAction,
        ),
        With<Player>,
    >,
    node_pieces: Query<(Option<&Actions>, &IsTapped), With<NodePiece>>,
    grid_uis: Query<(), With<GridUi>>,
    teams: Query<&TeamPhase, With<Team>>,
    mut ev_node_op: EventWriter<Op<NodeOp>>,
    mut ev_node_ui_op: EventWriter<Op<NodeUiOp>>,
) {
    for KeyEvent { code, modifiers } in ev_keys.read() {
        for (
            player,
            InNode(node),
            OnTeam(team),
            UiFocus(focus_opt),
            key_map,
            cursor,
            selected_entity,
            selected_action,
        ) in players.iter()
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
                    let team_phase = teams.get(*team).ok()?;

                    match named_input {
                        NamedInput::Direction(dir) => {
                            if is_controlling_active_curio && selected_action.is_none() {
                                // ev_node_op.send(NodeOp::MoveActiveCurio { dir }.for_p(player));
                                // res_prime_op_queue.push_back(NodeOp::MoveActiveCurio { dir }.to_request(player));
                                res_prime_ops.request(player, NodeOp::MoveActiveCurio { dir })
                            } else {
                                ev_node_ui_op
                                    .send(NodeUiOp::MoveNodeCursor(dir.into()).for_p(player))
                            }
                        },
                        NamedInput::Activate => {
                            if let Some(selected_action_index) = **selected_action {
                                selected_entity.of(&node_pieces).and_then(
                                    |(actions, is_tapped)| {
                                        if **is_tapped || *team_phase == TeamPhase::Setup {
                                            return None;
                                        }
                                        let action = ast_actions.get(actions?.get(selected_action_index)?)?;
                                        res_prime_ops.request(
                                            player,
                                            NodeOp::PerformCurioAction {
                                                action_id: action.id_cow(),
                                                curio: **selected_entity,
                                                target: **cursor,
                                            },
                                        );

                                        Some(())
                                    },
                                );
                            } else if is_controlling_active_curio {
                                selected_entity.of(&node_pieces).and_then(
                                    |(actions, is_tapped)| {
                                        if **is_tapped {
                                            return None;
                                        }
                                        match actions.map(|actions| (actions.len(), actions)) {
                                            None | Some((0, _)) => {
                                                res_prime_ops.request(
                                                    player,
                                                    NodeOp::PerformCurioAction {
                                                        action_id: NO_OP_ACTION_ID,
                                                        curio: **selected_entity,
                                                        target: default(),
                                                    },
                                                );
                                            },
                                            Some((1, actions)) => {
                                                if let Some(action) = ast_actions.get(actions.0.get(0).expect("if the len is 1, there should be an action at 0")) {
                                                    if action.range().is_none() {
                                                        res_prime_ops.request(
                                                            player,
                                                            NodeOp::PerformCurioAction {
                                                                action_id: action.id_cow(),
                                                                curio: **selected_entity,
                                                                target: default(),
                                                            },
                                                        );
                                                    }
                                                }
                                            },
                                            _ => {},
                                        }
                                        Some(())
                                    },
                                );
                            // If the curio has an action menu, focus on it
                            } else if let Some(curio_id) = **selected_entity {
                                if **turn == *team && *team_phase != TeamPhase::Setup {
                                    res_prime_ops.request(player, NodeOp::ActivateCurio { curio_id });
                                }
                            }
                        },
                        NamedInput::Undo => {
                            if selected_action.is_some() {
                                NodeUiOp::SetSelectedAction(None)
                                    .for_p(player)
                                    .send(&mut ev_node_ui_op);
                                if is_controlling_active_curio {
                                    active_curio.and_then(|active_curio_id| {
                                        let head = grid.head(active_curio_id)?;
                                        NodeUiOp::MoveNodeCursor(head.into())
                                            .for_p(player)
                                            .send(&mut ev_node_ui_op);
                                        Some(())
                                    });
                                }
                            }
                        },
                        _ => {},
                    }
                    Some(())
                });
        }
    }
}
