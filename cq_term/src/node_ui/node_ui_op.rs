use game_core::node::{CurrentTurn, InNode, Node, OnTeam, Team, TeamPhase};
use game_core::op::{Op, OpError, OpErrorUtils, OpExecutor, OpImplResult};
use game_core::player::{ForPlayer, Player};

use super::grid_ui::GridUi;
use super::menu_ui::{MenuUiActions, MenuUiCardSelection};
use super::{CursorIsHidden, NodeCursor, SelectedAction, SelectedEntity};
use crate::layout::{
    ui_focus_cycle_next, ui_focus_cycle_prev, StyleTty, UiFocus, UiFocusCycleOrder,
};
use crate::prelude::*;

#[derive(Debug, Default, Deref, DerefMut, Resource)]
pub struct UiOps(OpExecutor);

#[derive(Clone, Debug, Reflect)]
pub enum NodeUiOp {
    ChangeFocus(FocusTarget),
    MoveNodeCursor(CompassOrPoint),
    SetCursorHidden(bool),
    SetSelectedAction(Option<usize>),
}

#[derive(Clone, Copy, Debug, Reflect)]
pub enum FocusTarget {
    Next,
    Prev,
    Grid,
    CardMenu,
    ActionMenu,
}

impl Op for NodeUiOp {
    fn register_systems(mut registrar: game_core::op::OpRegistrar<Self>)
    where
        Self: Sized + bevy::prelude::TypePath + FromReflect,
    {
        registrar
            .register_op(opsys_nodeui_focus)
            .register_op(opsys_nodeui_move_cursor)
            .register_op(opsys_nodeui_hide_cursor)
            .register_op(opsys_nodeui_selected_action);
    }

    fn system_index(&self) -> usize {
        match self {
            Self::ChangeFocus(_) => 0,
            Self::MoveNodeCursor(_) => 1,
            Self::SetCursorHidden(_) => 2,
            Self::SetSelectedAction(_) => 3,
        }
    }
}

pub fn opsys_nodeui_focus(
    In((player, op)): In<(Entity, NodeUiOp)>,
    ui_nodes: Query<(Entity, &StyleTty, &UiFocusCycleOrder, &ForPlayer)>,
    action_menus: IndexedQuery<ForPlayer, Entity, With<MenuUiActions>>,
    grid_uis: IndexedQuery<ForPlayer, Entity, With<GridUi>>,
    card_selection_menus: IndexedQuery<ForPlayer, Entity, With<MenuUiCardSelection>>,
    mut players: Query<(AsDerefMut<UiFocus>, AsDerefMut<CursorIsHidden>), With<Player>>,
) -> OpImplResult {
    if let NodeUiOp::ChangeFocus(focus_target) = op {
        let (mut focus, mut cursor_is_hidden) = players.get_mut(player).critical()?;
        let next_focus = match focus_target {
            FocusTarget::Next => ui_focus_cycle_next(*focus, player, 0, &ui_nodes),
            FocusTarget::Prev => ui_focus_cycle_prev(*focus, player, 0, &ui_nodes),
            FocusTarget::ActionMenu => action_menus.get_for(player).ok(),
            FocusTarget::Grid => grid_uis.get_for(player).ok(),
            FocusTarget::CardMenu => card_selection_menus.get_for(player).ok(),
        };
        focus.set_if_neq(next_focus);
        cursor_is_hidden.set_if_neq(false);
        Ok(default())
    } else {
        Err(OpError::MismatchedOpSystem)
    }
}

pub fn opsys_nodeui_selected_action(
    In((player, op)): In<(Entity, NodeUiOp)>,
    mut players: Query<(AsDerefMut<SelectedAction>,), With<Player>>,
) -> OpImplResult {
    if let NodeUiOp::SetSelectedAction(next_selected_action) = op {
        let (mut selected_action,) = players.get_mut(player).critical()?;
        selected_action.set_if_neq(next_selected_action);
        Ok(default())
    } else {
        Err(OpError::MismatchedOpSystem)
    }
}

pub fn opsys_nodeui_move_cursor(
    In((player, op)): In<(Entity, NodeUiOp)>,
    mut players: Query<(&InNode, AsDerefMut<NodeCursor>, AsDerefMut<CursorIsHidden>), With<Player>>,
    nodes: Query<(&EntityGrid,), With<Node>>,
) -> OpImplResult {
    if let NodeUiOp::MoveNodeCursor(compass_or_point) = op {
        let (InNode(node), mut cursor, mut cursor_is_hidden) =
            players.get_mut(player).critical()?;
        let (grid,) = nodes.get(*node).critical()?;
        let next_pt = grid
            .index_bounds()
            .min(compass_or_point.point_from(*cursor));
        cursor.set_if_neq(next_pt);
        cursor_is_hidden.set_if_neq(false);
        Ok(default())
    } else {
        Err(OpError::MismatchedOpSystem)
    }
}

pub fn opsys_nodeui_hide_cursor(
    In((player, op)): In<(Entity, NodeUiOp)>,
    mut players: Query<AsDerefMut<CursorIsHidden>, With<Player>>,
) -> OpImplResult {
    if let NodeUiOp::SetCursorHidden(val) = op {
        players.get_mut(player).critical()?.set_if_neq(val);
        Ok(default())
    } else {
        Err(OpError::MismatchedOpSystem)
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

pub fn sys_adjust_selected_action(
    mut players: Query<(&UiFocus, &mut SelectedAction), (Changed<UiFocus>, With<Player>)>,
    action_menus: Query<(), With<MenuUiActions>>,
) {
    for (ui_focus, mut selected_action) in players.iter_mut() {
        if ui_focus
            .map(|focus| action_menus.contains(focus))
            .unwrap_or(false)
            && selected_action.is_none()
        {
            **selected_action = Some(0);
        }
    }
}
