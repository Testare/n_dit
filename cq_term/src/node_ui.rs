mod button_ui;
mod grid_ui;
mod inputs;
mod menu_ui;
mod messagebar_ui;
mod node_context_actions;
mod node_glyph;
mod node_popups;
mod node_ui_op;
mod setup;
mod titlebar_ui;

use bevy::ecs::query::{QueryData, QueryFilter, WorldQuery};
use bevy::reflect::Reflect;
use game_core::board::BoardScreen;
use game_core::card::{Action, Actions};
use game_core::node::{
    self, ActiveCurio, Curio, CurrentTurn, EnteringNode, InNode, Node, NodeOp, OnTeam,
};
use game_core::op::{OpPlugin, OpResult};
use game_core::player::{ForPlayer, Player};
use game_core::registry::Reg;
use game_core::NDitCoreSet;

use self::grid_ui::GridUi;
use self::menu_ui::{
    MenuUiActions, MenuUiCardSelection, MenuUiDescription, MenuUiLabel, MenuUiStats,
};
pub use self::messagebar_ui::MessageBarUi;
pub use self::node_glyph::NodeGlyph;
pub use self::node_ui_op::NodeUiOp;
use self::titlebar_ui::TitleBarUi;
use super::layout::StyleTty;
use super::render::TerminalRendering;
use crate::main_ui::{MainUiOp, UiOps};
use crate::node_ui::node_ui_op::FocusTarget;
use crate::prelude::*;
use crate::{KeyMap, Submap};
/// Plugin for NodeUI
#[derive(Debug, Default)]
pub struct NodeUiPlugin;

impl Plugin for NodeUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((Reg::<NodeGlyph>::default(), OpPlugin::<NodeUiOp>::default()))
            .init_resource::<setup::ButtonContextActions>()
            .init_resource::<node_context_actions::NodeContextActions>()
            .add_systems(Update, (setup::create_node_ui, sys_switch_screens_on_enter))
            .add_systems(
                PreUpdate,
                (inputs::kb_ready, inputs::kb_skirm_focus).in_set(NDitCoreSet::ProcessInputs),
            )
            .add_systems(
                Update,
                (
                    (sys_react_to_node_op, button_ui::sys_undo_button_state)
                        .in_set(NDitCoreSet::PostProcessCommands),
                    (
                        node_ui_op::sys_adjust_selected_action,
                        node_ui_op::sys_adjust_selected_entity,
                        button_ui::sys_ready_button_disable,
                    )
                        .chain()
                        .in_set(NDitCoreSet::PostProcessUiOps),
                ),
            )
            .add_plugins((
                MenuUiCardSelection::plugin(),
                MenuUiStats::plugin(),
                MenuUiLabel::plugin(),
                MenuUiActions::plugin(),
                MenuUiDescription::plugin(),
                GridUi::plugin(),
                MessageBarUi::plugin(),
                TitleBarUi::plugin(),
            ));
    }
}

#[derive(Component, Debug, Default)]
pub struct NodeUiScreen;

#[derive(Component, Debug)]
pub struct HasNodeUi;

/// Component that tells the UI which node piece the node cursor is over
///
/// Stored on Player entity
#[derive(Component, Resource, Debug, Deref, DerefMut, PartialEq)]
pub struct SelectedNodePiece(pub Option<Entity>);

#[derive(Component, Debug, Deref, DerefMut, PartialEq)]
pub struct SelectedAction(Option<usize>);

#[derive(Component, Debug, Deref, DerefMut)]
pub struct TelegraphedAction(Option<Handle<Action>>);

#[derive(Component, Debug, Default, Deref, DerefMut)]
pub struct AvailableMoves(HashMap<UVec2, Option<Compass>>);

#[derive(Component, Debug, Default, Deref, DerefMut)]
pub struct AvailableActionTargets(HashMap<UVec2, bool>);

/// Cursor that the user controls to select pieces in the node
#[derive(Component, Debug, Default, Deref, DerefMut, Reflect)]
pub struct NodeCursor(pub UVec2);

#[derive(Component, Debug, Default, Deref, DerefMut, Reflect)]
pub struct CursorIsHidden(pub bool);

impl SelectedNodePiece {
    pub fn of<'a, Q: QueryData, R: QueryFilter>(
        &self,
        query: &'a Query<Q, R>,
    ) -> Option<<<Q as QueryData>::ReadOnly as WorldQuery>::Item<'a>> {
        query.get(self.0?).ok()
    }

    pub fn of_mut<'a, Q: QueryData, R: QueryFilter>(
        &self,
        query: &'a mut Query<Q, R>,
    ) -> Option<<Q as WorldQuery>::Item<'a>> {
        query.get_mut(self.0?).ok()
    }
}

#[derive(Debug, QueryData)]
pub struct NodeUiQ {
    grid: &'static EntityGrid,
}

pub trait NodeUi: Component + Default {
    const NAME: &'static str;
    type UiBundleExtras: Bundle;
    type UiPlugin: Plugin + Default;

    fn ui_bundle_extras() -> Self::UiBundleExtras;
    fn initial_style(node_q: &NodeUiQItem) -> StyleTty;

    fn bundle(
        player: Entity,
        node_q: &NodeUiQItem,
    ) -> (
        StyleTty,
        Name,
        ForPlayer,
        Self::UiBundleExtras,
        Self,
        TerminalRendering,
    ) {
        (
            Self::initial_style(node_q),
            Name::new(Self::NAME),
            ForPlayer(player),
            Self::ui_bundle_extras(),
            Self::default(),
            TerminalRendering::default(),
        )
    }

    fn plugin() -> Self::UiPlugin {
        Self::UiPlugin::default()
    }
}

pub fn sys_switch_screens_on_enter(
    mut res_ui_ops: ResMut<UiOps>,
    mut q_removed_entering_node: RemovedComponents<EnteringNode>,
    mut q_player: Query<&mut KeyMap, (With<HasNodeUi>, With<Player>)>,
    q_node_ui_screen: Query<(Entity, &ForPlayer), With<NodeUiScreen>>,
) {
    for player_id in q_removed_entering_node.read() {
        if let Ok(mut key_map) = q_player.get_mut(player_id) {
            for (node_ui_screen_id, &ForPlayer(nui_player_id)) in q_node_ui_screen.iter() {
                if nui_player_id == player_id {
                    res_ui_ops.request(player_id, MainUiOp::SwitchScreen(node_ui_screen_id));
                    key_map.activate_submap(Submap::Node);
                }
            }
        }
    }
}

// This should be broken up into separate systems
fn sys_react_to_node_op(
    ast_action: Res<Assets<Action>>,
    mut evr_op_result: EventReader<OpResult<NodeOp>>,
    mut res_ui_ops: ResMut<UiOps>,
    q_node: Query<
        (
            &EntityGrid,
            AsDerefCopied<CurrentTurn>,
            AsDerefCopied<ActiveCurio>,
        ),
        With<Node>,
    >,
    q_player_with_node_ui: Query<(), (With<Player>, With<HasNodeUi>)>,
    q_player_in_node: Query<AsDerefCopied<InNode>, With<Player>>,
    q_board_screen: Query<(AsDerefCopied<ForPlayer>, Entity), With<BoardScreen>>,
    mut q_player_key_map: Query<&mut KeyMap, With<Player>>,
    mut q_player_ui: Query<
        (
            Entity,
            AsDerefCopied<OnTeam>,
            AsDerefCopied<InNode>,
            &mut TelegraphedAction,
        ),
        (With<Player>, With<HasNodeUi>),
    >,
    q_curio: Query<&Actions, With<Curio>>,
) {
    for op_result in evr_op_result.read() {
        // Reactions to ops from other players in node
        if op_result.result().is_ok() {
            q_player_in_node
                .get(op_result.source())
                .ok()
                .and_then(|node| {
                    match op_result.op() {
                        NodeOp::EndTurn => {
                            let (_, current_turn, _) = get_assert!(node, q_node)?;
                            for (id, team, _, _) in q_player_ui.iter() {
                                if team == current_turn {
                                    res_ui_ops.request(id, NodeUiOp::SetCursorHidden(false));
                                }
                            }
                        },
                        NodeOp::TelegraphAction { action_id } => {
                            let (_, _, active_curio) = get_assert!(node, q_node)?;
                            let actions = q_curio.get(active_curio?).ok()?;
                            let action_handle = actions.iter().find_map(|action_handle| {
                                let action_def = ast_action.get(action_handle)?;
                                (action_def.id() == action_id).then_some(action_handle.clone())
                            });

                            for (_, _, in_node, mut telegraphed_action) in q_player_ui.iter_mut() {
                                if in_node == node {
                                    **telegraphed_action = action_handle.clone();
                                }
                            }
                        },
                        NodeOp::PerformCurioAction { .. } => {
                            for (_, _, in_node, mut telegraphed_action) in q_player_ui.iter_mut() {
                                if in_node == node {
                                    **telegraphed_action = None;
                                }
                            }
                        },
                        _ => {},
                    }
                    Some(())
                });
        }
        if !q_player_with_node_ui.contains(op_result.source()) {
            continue;
        }

        // Reactions to own actions
        if let Ok(metadata) = op_result.result() {
            let player = op_result.source();
            match op_result.op() {
                NodeOp::MoveActiveCurio { .. } => {
                    // NOTE this will probably fail when an AI takes an action
                    get_assert!(player, q_player_in_node, |node| {
                        let (grid, _, _) = get_assert!(node, q_node)?;
                        let curio = metadata.get_required(node::key::CURIO).ok()?;
                        let remaining_moves =
                            metadata.get_required(node::key::REMAINING_MOVES).ok()?;
                        let tapped = metadata.get_or_default(node::key::TAPPED).ok()?;
                        res_ui_ops
                            .request(player, NodeUiOp::MoveNodeCursor(grid.head(curio)?.into()));
                        if remaining_moves == 0 && !tapped {
                            res_ui_ops
                                .request(player, NodeUiOp::ChangeFocus(FocusTarget::ActionMenu));
                        }
                        Some(())
                    });
                },
                NodeOp::Undo => {
                    get_assert!(player, q_player_in_node, |node| {
                        // Should I use IN_NODE metadata instead?
                        let (grid, _, _) = get_assert!(node, q_node)?;
                        let curio = metadata.get_optional(node::key::CURIO).ok()??;
                        res_ui_ops
                            .request(player, NodeUiOp::MoveNodeCursor(grid.head(curio)?.into()));
                        res_ui_ops.request(player, NodeUiOp::ChangeFocus(FocusTarget::Grid));
                        res_ui_ops.request(player, NodeUiOp::SetSelectedAction(None));
                        Some(())
                    });
                },
                NodeOp::ReadyToGo | NodeOp::PerformCurioAction { .. } => {
                    res_ui_ops.request(player, NodeUiOp::ChangeFocus(FocusTarget::Grid));
                    res_ui_ops.request(player, NodeUiOp::SetSelectedAction(None));
                },
                NodeOp::EndTurn => {
                    res_ui_ops.request(player, NodeUiOp::ChangeFocus(FocusTarget::Grid));
                    res_ui_ops.request(player, NodeUiOp::SetCursorHidden(true));
                },
                NodeOp::QuitNode(_) => {
                    res_ui_ops.request(player, NodeUiOp::SetSelectedAction(None));
                    if let Some((_, board_screen_id)) = q_board_screen
                        .iter()
                        .find(|&(i_player_id, _)| i_player_id == player)
                    {
                        res_ui_ops.request(player, MainUiOp::SwitchScreen(board_screen_id));
                        if let Ok(mut key_map) = q_player_key_map.get_mut(player) {
                            key_map.deactivate_submap(Submap::Node)
                        }
                    }
                },
                _ => {},
            }
        }
    }
}
