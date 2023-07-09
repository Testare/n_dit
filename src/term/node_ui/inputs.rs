use game_core::card::Actions;
use game_core::node::{
    AccessPoint, ActiveCurio, CurrentTurn, InNode, Node, NodeOp, NodePiece, OnTeam,
};
use game_core::player::{ForPlayer, Player};

use super::grid_ui::GridUi;
use super::menu_ui::{MenuUiActions, MenuUiCardSelection};
use super::{SelectedAction, SelectedEntity};
use crate::term::key_map::NamedInput;
use crate::term::layout::{
    ui_focus_cycle_next, ui_focus_cycle_prev, StyleTty, UiFocus, UiFocusCycleOrder, UiFocusNext,
};
use crate::term::prelude::*;
use crate::term::{KeyMap, Submap};

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
                        NamedInput::AltActivate => {
                            if **focus == card_menu_id {
                                **focus_next = grid_id;
                            }
                        },
                        _ => {},
                    }
                    Some(())
                });
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
