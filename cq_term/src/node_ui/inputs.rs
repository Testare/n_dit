use game_core::card::Actions;
use game_core::node::{
    AccessPoint, ActiveCurio, CurrentTurn, InNode, Node, NodeOp, NodePiece, OnTeam,
};
use game_core::op::CoreOps;
use game_core::player::Player;

use super::grid_ui::GridUi;
use super::menu_ui::{MenuUiActions, MenuUiCardSelection};
use super::node_ui_op::{FocusTarget, UiOps};
use super::{NodeUiOp, SelectedAction, SelectedNodePiece};
use crate::key_map::NamedInput;
use crate::layout::UiFocus;
use crate::prelude::*;
use crate::{KeyMap, Submap};

pub fn kb_ready(
    mut res_prime_op: ResMut<CoreOps>,
    mut players: Query<(Entity, &KeyMap), (With<Player>, With<InNode>)>,
    mut ev_keys: EventReader<KeyEvent>,
) {
    for KeyEvent { code, modifiers } in ev_keys.read() {
        for (player, key_map) in players.iter_mut() {
            if matches!(
                key_map.named_input_for_key(Submap::Node, *code, *modifiers),
                Some(NamedInput::Ready)
            ) {
                res_prime_op.request(player, NodeOp::ReadyToGo);
            }
        }
    }
}

pub fn kb_skirm_focus(
    mut res_ui_ops: ResMut<UiOps>,
    mut ev_keys: EventReader<KeyEvent>,
    players: Query<
        (
            Entity,
            &InNode,
            &OnTeam,
            &UiFocus,
            &KeyMap,
            &SelectedNodePiece,
            &SelectedAction,
        ),
        With<Player>,
    >,
    nodes: Query<(&ActiveCurio, &CurrentTurn), With<Node>>,
    action_pieces: Query<&Actions, With<NodePiece>>,
    access_points: Query<(), (With<AccessPoint>, With<NodePiece>)>,
    skirm_uis: Query<(), Or<(With<GridUi>, With<MenuUiCardSelection>, With<MenuUiActions>)>>,
    grid_uis: Query<(), With<GridUi>>,
    card_menus: Query<(), With<MenuUiCardSelection>>,
    action_menus: Query<(), With<MenuUiActions>>,
) {
    for KeyEvent { code, modifiers } in ev_keys.read() {
        for (player, in_node, team, focus, key_map, selected_entity, selected_action) in
            players.iter()
        {
            if (**focus)
                .map(|focused_ui| !skirm_uis.contains(focused_ui))
                .unwrap_or(false)
            {
                continue;
            }

            let active_curio = get_assert!(**in_node, nodes, |(active_curio, turn)| {
                if **turn == **team {
                    **active_curio
                } else {
                    None
                }
            });

            if let Some(focus_target) = key_map
                .named_input_for_key(Submap::Node, *code, *modifiers)
                .and_then(|named_input| {
                    match named_input {
                        NamedInput::Activate => {
                            // Activate on an access_point => Focus card selection menu
                            if (focus.map(|focus| grid_uis.contains(focus)).unwrap_or(true))
                                && selected_entity.of(&access_points).is_some()
                            {
                                Some(FocusTarget::CardMenu)
                            } else if focus
                                .map(|focus| action_menus.contains(focus))
                                .unwrap_or_default()
                            {
                                Some(FocusTarget::Grid)
                            } else if let Some(actions) =
                                active_curio.and_then(|curio_id| action_pieces.get(curio_id).ok())
                            {
                                if actions.len() > 1 && selected_action.is_none() {
                                    Some(FocusTarget::ActionMenu)
                                } else {
                                    None
                                }
                            } else {
                                None
                            }
                        },
                        NamedInput::AltActivate => {
                            if focus
                                .map(|focus| card_menus.contains(focus))
                                .unwrap_or_default()
                            {
                                Some(FocusTarget::Grid)
                            } else {
                                None
                            }
                        },
                        NamedInput::MenuFocusNext => Some(FocusTarget::Next),
                        NamedInput::MenuFocusPrev => Some(FocusTarget::Prev),
                        NamedInput::Undo => Some(FocusTarget::Grid),
                        _ => None,
                    }
                })
            {
                res_ui_ops.request(player, NodeUiOp::ChangeFocus(focus_target));
            }
        }
    }
}
