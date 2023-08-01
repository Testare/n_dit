use std::ops::Deref;

use crossterm::event::KeyModifiers;
use game_core::card::{Action, ActionRange, Actions};
use game_core::node::{
    ActiveCurio, Curio, CurrentTurn, InNode, IsTapped, NoOpAction, Node, NodeOp, NodePiece, OnTeam,
    Pickup, Team, TeamPhase,
};
use game_core::player::{ForPlayer, Player};

use super::{GridUi, Scroll2D};
use crate::input_event::{MouseButton, MouseEventKind};
use crate::key_map::NamedInput;
use crate::layout::{LayoutEvent, UiFocus};
use crate::node_ui::{NodeCursor, SelectedAction, SelectedEntity};
use crate::prelude::*;
use crate::{KeyMap, Submap};

pub fn handle_layout_events(
    mut ev_mouse: EventReader<LayoutEvent>,
    ui: Query<(&ForPlayer, &Scroll2D), With<GridUi>>,
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
    nodes: Query<(&EntityGrid, &ActiveCurio, &CurrentTurn), With<Node>>,
    teams: Query<&TeamPhase, With<Team>>,
    pickups: Query<(), With<Pickup>>,
    curios: Query<(&OnTeam, &Actions, &IsTapped), With<Curio>>,
    actions: Query<(&ActionRange,), With<Action>>,
    mut ev_node_op: EventWriter<Op<NodeOp>>,
) {
    for event in ev_mouse.iter() {
        if let Ok((ForPlayer(player), scroll)) = ui.get(event.entity()) {
            if let MouseEventKind::Down(button) = event.event_kind() {
                log::trace!("Clicked on the grid");
                get_assert_mut!(*player, players, |(
                    node,
                    team,
                    mut cursor,
                    mut selected_entity,
                    mut selected_action,
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
                        if **cursor.deref() != clicked_node_pos {
                            **cursor = clicked_node_pos;
                        }

                        let now_selected_entity = grid.item_at(**cursor);
                        if selected_entity.0 != now_selected_entity
                            && (now_selected_entity.is_some() || !is_controlling_active_curio)
                        {
                            selected_entity.0 = now_selected_entity;
                            **selected_action = None;
                        }
                    } else {
                        let selected_action = selected_action.and_then(|selected_action| {
                            let (_, actions, tapped) = selected_entity.of(&curios)?;
                            if **tapped {
                                return None;
                            }
                            actions.get(selected_action).copied()
                        });
                        let pt_in_range = selected_action
                            .as_ref()
                            .and_then(|selected_action| {
                                let (range,) = actions.get(*selected_action).ok()?;
                                Some(range.in_range(
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
                                    action,
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
    no_op_action: Res<NoOpAction>,
    nodes: Query<(&EntityGrid, &ActiveCurio, &CurrentTurn), With<Node>>,
    mut players: Query<
        (
            Entity, // Solid candidate for making a WorldQuery derive
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
    node_pieces: Query<(Option<&Actions>, &IsTapped), With<NodePiece>>,
    mut ev_keys: EventReader<KeyEvent>,
    mut ev_node_op: EventWriter<Op<NodeOp>>,
    grid_uis: Query<(), With<GridUi>>,
    rangeless_actions: Query<(), (Without<ActionRange>, With<Action>)>,
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
            mut selected_action,
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
                            if selected_action.is_some() {
                                // Do not adjust selected entity/action
                                **cursor = (**cursor + dir).min(grid.index_bounds());
                            } else if is_controlling_active_curio {
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
                        NamedInput::Activate => {
                            if let Some(selected_action_index) = **selected_action {
                                selected_entity.of(&node_pieces).and_then(
                                    |(actions, is_tapped)| {
                                        if **is_tapped {
                                            return None;
                                        }
                                        let action = *actions?.get(selected_action_index)?;
                                        ev_node_op.send(Op::new(
                                            player,
                                            NodeOp::PerformCurioAction {
                                                action,
                                                curio: **selected_entity,
                                                target: **cursor,
                                            },
                                        ));
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
                                                ev_node_op.send(Op::new(
                                                    player,
                                                    NodeOp::PerformCurioAction {
                                                        action: **no_op_action,
                                                        curio: **selected_entity,
                                                        target: default(),
                                                    },
                                                ));
                                            },
                                            Some((1, actions)) => {
                                                let action = *actions.0.get(0).expect(
                                                "if the len is 1, there should be an action at 0",
                                            );
                                                if rangeless_actions.contains(action) {
                                                    ev_node_op.send(Op::new(
                                                        player,
                                                        NodeOp::PerformCurioAction {
                                                            action,
                                                            curio: **selected_entity,
                                                            target: default(),
                                                        },
                                                    ));
                                                } else {
                                                    **selected_action = Some(0);
                                                }
                                            },
                                            _ => {},
                                        }
                                        Some(())
                                    },
                                );
                                // If the curio has an action menu, focus on it
                            } else if let Some(curio_id) = **selected_entity {
                                ev_node_op
                                    .send(Op::new(player, NodeOp::ActivateCurio { curio_id }));
                            }
                        },
                        NamedInput::Undo => {
                            if selected_action.is_some() {
                                **selected_action = None;
                                if is_controlling_active_curio {
                                    active_curio.and_then(|active_curio_id| {
                                        **cursor = grid.head(active_curio_id)?;
                                        Some(())
                                    });
                                }
                                cursor.adjust_to_self(selected_entity, selected_action, grid);
                            }
                        },
                        _ => {},
                    }
                    Some(())
                });
        }
    }
}
