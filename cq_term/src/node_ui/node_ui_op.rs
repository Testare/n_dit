use game_core::node::{CurrentTurn, InNode, Node, OnTeam, Team, TeamPhase};
use game_core::op::OpSubtype;
use game_core::player::{ForPlayer, Player};

use super::grid_ui::GridUi;
use super::menu_ui::{MenuUiActions, MenuUiCardSelection};
use super::{NodeCursor, SelectedAction, SelectedEntity};
use crate::layout::{
    ui_focus_cycle_next, ui_focus_cycle_prev, StyleTty, UiFocus, UiFocusCycleOrder, UiFocusNext,
};
use crate::prelude::*;

#[derive(Clone, Debug)]
pub enum NodeUiOp {
    ChangeFocus(FocusTarget),
    MoveNodeCursor(CompassOrPoint),
    SetSelectedAction(Option<usize>),
}

#[derive(Clone, Copy, Debug)]
pub enum FocusTarget {
    Next,
    Prev,
    Grid,
    CardMenu,
    ActionMenu,
}

impl OpSubtype for NodeUiOp {
    type Error = ();
}

pub fn sys_node_ui_op_change_focus(
    mut ev_node_ui_op: EventReader<Op<NodeUiOp>>,
    ui_nodes: Query<(Entity, &StyleTty, &UiFocusCycleOrder, &ForPlayer)>,
    action_menus: IndexedQuery<ForPlayer, Entity, With<MenuUiActions>>,
    grid_uis: IndexedQuery<ForPlayer, Entity, With<GridUi>>,
    card_selection_menus: IndexedQuery<ForPlayer, Entity, With<MenuUiCardSelection>>,
    mut players: Query<(&UiFocus, &mut UiFocusNext), With<Player>>,
) {
    for Op { player, op } in ev_node_ui_op.iter() {
        if let NodeUiOp::ChangeFocus(focus_target) = op {
            get_assert_mut!(*player, &mut players, |(focus, mut focus_next)| {
                let next_focus = match focus_target {
                    FocusTarget::Next => ui_focus_cycle_next(**focus, *player, 0, &ui_nodes),
                    FocusTarget::Prev => ui_focus_cycle_prev(**focus, *player, 0, &ui_nodes),
                    FocusTarget::ActionMenu => action_menus.get_for(*player).ok(),
                    FocusTarget::Grid => grid_uis.get_for(*player).ok(),
                    FocusTarget::CardMenu => card_selection_menus.get_for(*player).ok(),
                };
                if **focus != next_focus {
                    **focus_next = next_focus
                }
                Some(())
            });
        }
    }
}

pub fn sys_node_ui_op_set_selected_action(
    mut ev_node_ui_op: EventReader<Op<NodeUiOp>>,
    mut players: Query<(&mut SelectedAction,), With<Player>>,
) {
    for Op { player, op } in ev_node_ui_op.iter() {
        if let NodeUiOp::SetSelectedAction(next_selected_action) = op {
            get_assert_mut!(*player, &mut players, |(mut selected_action,)| {
                **selected_action = next_selected_action.clone();
                Some(())
            });
        }
    }
}

pub fn sys_node_ui_op_move_cursor(
    mut ev_node_ui_op: EventReader<Op<NodeUiOp>>,
    mut players: Query<(&InNode, &mut NodeCursor), With<Player>>,
    nodes: Query<(&EntityGrid,), With<Node>>,
) {
    for Op { player, op } in ev_node_ui_op.iter() {
        if let NodeUiOp::MoveNodeCursor(compass_or_point) = op {
            get_assert_mut!(*player, &mut players, |(InNode(node), mut cursor)| {
                let (grid,) = get_assert!(*node, nodes)?;
                let next_pt = grid
                    .index_bounds()
                    .min(compass_or_point.point_from(**cursor));
                if **cursor != next_pt {
                    **cursor = next_pt;
                }
                Some(())
            });
        }
    }
}

pub fn sys_adjust_selected_entity(
    mut players: Query<
        (
            &InNode,
            &OnTeam,
            &NodeCursor,
            &mut SelectedAction,
            &mut SelectedEntity,
        ),
        (
            With<Player>,
            Or<(Changed<NodeCursor>, Changed<SelectedAction>)>,
        ),
    >,
    nodes: Query<(&EntityGrid, &CurrentTurn), With<Node>>,
    teams: Query<(&TeamPhase,), With<Team>>,
) {
    for (in_node, on_team, cursor, mut selected_action, mut selected_entity) in players.iter_mut() {
        get_assert!(**in_node, nodes, |(grid, current_turn)| {
            let (team_phase,) = get_assert!(**on_team, teams)?;
            if selected_action.is_none() {
                **selected_entity = grid.item_at(**cursor);
            } else if **on_team != **current_turn || *team_phase != TeamPhase::Play {
                let entity_at_cursor = grid.item_at(**cursor);
                if **selected_entity != entity_at_cursor {
                    **selected_entity = grid.item_at(**cursor);
                    **selected_action = None;
                }
            }
            Some(())
        });
    }
}
