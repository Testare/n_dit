use game_core::card::{Action, ActionRange, Actions};
use game_core::node::{
    AccessPoint, ActiveCurio, CurrentTurn, InNode, NoOpAction, Node, NodeOp, NodePiece, OnTeam,
};
use game_core::player::{ForPlayer, Player};

use super::grid_ui::GridUi;
use super::menu_ui::{MenuUiActions, MenuUiCardSelection};
use super::{MessageBarUi, NodeCursor, SelectedAction, SelectedEntity};
use crate::term::key_map::NamedInput;
use crate::term::layout::{
    ui_focus_cycle_next, ui_focus_cycle_prev, StyleTty, UiFocus, UiFocusCycleOrder, UiFocusNext,
};
use crate::term::prelude::*;
use crate::term::{KeyMap, Submap};

pub fn kb_messages(
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

pub fn kb_ready(
    mut players: Query<(Entity, &KeyMap), (With<Player>, With<InNode>)>,
    mut ev_keys: EventReader<KeyEvent>,
    mut ev_node_op: EventWriter<Op<NodeOp>>,
) {
    for KeyEvent { code, modifiers } in ev_keys.iter() {
        for (player, key_map) in players.iter_mut() {
            key_map
                .named_input_for_key(Submap::Node, *code, *modifiers)
                .and_then(|named_input| {
                    if matches!(named_input, NamedInput::Ready) {
                        ev_node_op.send(Op::new(player, NodeOp::ReadyToGo));
                    }
                    Some(())
                });
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
    node_pieces: Query<(Option<&Actions>,), With<NodePiece>>,
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
                                selected_entity.of(&node_pieces).and_then(|(actions,)| {
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
                                });
                            } else if is_controlling_active_curio {
                                selected_entity.of(&node_pieces).and_then(|(actions,)| {
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
                                });
                                // If the curio has an action menu, focus on it
                            } else if let Some(curio_id) = **selected_entity {
                                ev_node_op
                                    .send(Op::new(player, NodeOp::ActivateCurio { curio_id }));
                            }
                        },
                        NamedInput::Undo => {
                            if is_controlling_active_curio && selected_action.is_some() {
                                **selected_action = None;
                                active_curio.and_then(|active_curio_id| {
                                    let head = grid.head(active_curio_id)?;
                                    cursor.adjust_to(head, selected_entity, selected_action, grid);
                                    Some(())
                                });
                            }
                        },
                        _ => {},
                    }
                    Some(())
                });
        }
    }
}

pub fn kb_skirm_focus(
    mut players: Query<
        (
            Entity,
            &InNode,
            &OnTeam,
            &UiFocus,
            &mut UiFocusNext,
            &KeyMap,
            &SelectedEntity,
            &SelectedAction,
        ),
        With<Player>,
    >,
    mut ev_keys: EventReader<KeyEvent>,
    skirm_uis: Query<
        (Entity, &ForPlayer),
        Or<(With<GridUi>, With<MenuUiCardSelection>, With<MenuUiActions>)>,
    >,
    nodes: Query<(&ActiveCurio, &CurrentTurn), With<Node>>,
    access_points: Query<(), (With<AccessPoint>, With<NodePiece>)>,
    action_pieces: Query<&Actions, With<NodePiece>>,
    grid_uis: Query<(Entity, &ForPlayer), With<GridUi>>,
    card_menus: Query<(Entity, &ForPlayer), With<MenuUiCardSelection>>,
    action_menus: Query<(Entity, &ForPlayer), With<MenuUiActions>>,
) {
    for KeyEvent { code, modifiers } in ev_keys.iter() {
        for (
            player,
            in_node,
            team,
            focus,
            mut focus_next,
            key_map,
            selected_entity,
            selected_action,
        ) in players.iter_mut()
        {
            if (**focus)
                .map(|focused_ui| !skirm_uis.contains(focused_ui))
                .unwrap_or(false)
            {
                continue;
            }

            let grid_id = grid_uis
                .iter()
                .find(|(_, fp)| ***fp == player)
                .map(|(id, _)| id);
            let card_menu_id = card_menus
                .iter()
                .find(|(_, fp)| ***fp == player)
                .map(|(id, _)| id);
            let action_menu_id = action_menus
                .iter()
                .find(|(_, fp)| ***fp == player)
                .map(|(id, _)| id);

            let active_curio = get_assert!(**in_node, nodes, |(active_curio, turn)| {
                if **turn == **team {
                    **active_curio
                } else {
                    None
                }
            });

            if grid_id.is_none() {
                log::error!("Missing Grid UI entity");
            }
            if card_menu_id.is_none() {
                log::error!("Missing Card Menu entity");
            }
            if action_menu_id.is_none() {
                log::error!("Missing Action Menu entity");
            }

            key_map
                .named_input_for_key(Submap::Node, *code, *modifiers)
                .and_then(|named_input| {
                    match named_input {
                        NamedInput::Undo => {
                            if **focus != grid_id {
                                **focus_next = grid_id
                            }
                        },
                        NamedInput::Activate => {
                            // Activate on an access_point => Focus card selection menu
                            if (focus.is_none() || **focus == grid_id)
                                && selected_entity.of(&access_points).is_some()
                            {
                                **focus_next = card_menu_id;
                            } else if **focus == action_menu_id {
                                **focus_next = grid_id;
                            } else if let Some(actions) =
                                active_curio.and_then(|curio_id| action_pieces.get(curio_id).ok())
                            {
                                if actions.len() > 1 && selected_action.is_none() {
                                    **focus_next = action_menu_id;
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

pub fn kb_action_menu(
    mut players: Query<
        (
            Entity,
            &UiFocus,
            &KeyMap,
            &SelectedEntity,
            &mut SelectedAction,
        ),
        With<Player>,
    >,
    node_pieces: Query<(&Actions,), With<NodePiece>>,
    mut ev_keys: EventReader<KeyEvent>,
    action_menu_uis: Query<(), With<MenuUiActions>>,
    mut ev_node_op: EventWriter<Op<NodeOp>>,
    rangeless_actions: Query<(), (Without<ActionRange>, With<Action>)>,
) {
    for KeyEvent { code, modifiers } in ev_keys.iter() {
        for (player_id, focus, key_map, selected_entity, mut selected_action) in players.iter_mut()
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
                            NamedInput::MenuFocusNext | NamedInput::MenuFocusPrev => {
                                **selected_action = None;
                            },
                            NamedInput::Activate => {
                                if let Some(action) =
                                    actions.get(selected_action.unwrap_or_default())
                                {
                                    if rangeless_actions.contains(*action) {
                                        ev_node_op.send(Op::new(
                                            player_id,
                                            NodeOp::PerformCurioAction {
                                                action: *action,
                                                curio: **selected_entity,
                                                target: default(),
                                            },
                                        ))
                                    }
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

pub fn kb_focus_cycle(
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
