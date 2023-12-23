mod available_moves;
mod borders;
mod grid_animation;
mod grid_inputs;
mod grid_tooltip;
mod range_of_action;
mod render_grid;
mod render_square;
mod scroll;

use bevy::ecs::query::{Has, WorldQuery};
use game_core::card::{Action, Actions, MovementSpeed};
use game_core::node::{
    self, AccessPoint, ActiveCurio, Curio, CurrentTurn, InNode, IsTapped, MovesTaken, Node, NodeOp,
    NodePiece, OnTeam, Pickup,
};
use game_core::op::OpResult;
use game_core::player::Player;
use game_core::NDitCoreSet;
pub use grid_animation::GridUiAnimation;

use super::node_ui_op::{FocusTarget, UiOps};
use super::{
    AvailableActionTargets, AvailableMoves, CursorIsHidden, HasNodeUi, NodeCursor, NodeUi,
    NodeUiOp, NodeUiQItem, SelectedAction, SelectedEntity, TelegraphedAction,
};
use crate::base_ui::{HoverPoint, Scroll2d, Tooltip};
use crate::input_event::MouseEventListener;
use crate::layout::{StyleTty, UiFocusOnClick};
use crate::prelude::*;
use crate::render::{RenderTtySet, RENDER_TTY_SCHEDULE};

#[derive(Component, Default)]
pub struct GridUi;

#[derive(WorldQuery)]
pub struct NodePieceQ {
    piece: &'static NodePiece,
    speed: Option<AsDerefCopied<MovementSpeed>>,
    moves_taken: Option<AsDerefCopied<MovesTaken>>,
    is_tapped: Option<AsDerefCopied<IsTapped>>,
    pickup: Option<&'static Pickup>,
    access_point: Option<&'static AccessPoint>,
    curio: Option<&'static Curio>,
    has_curio: Has<Curio>,
    actions: Option<AsDeref<Actions>>,
    team: Option<AsDerefCopied<OnTeam>>,
}

#[derive(WorldQuery)]
pub struct PlayerUiQ {
    entity: Entity,
    selected_entity: &'static SelectedEntity,
    selected_action: &'static SelectedAction,
    telegraphed_action: &'static TelegraphedAction,
    node_cursor: &'static NodeCursor,
    cursor_is_hidden: AsDerefCopied<CursorIsHidden>,
    available_moves: &'static AvailableMoves,
    available_action_targets: &'static AvailableActionTargets,
    in_node: &'static InNode,
}

impl Plugin for GridUi {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PreUpdate,
            (grid_inputs::handle_layout_events, grid_inputs::kb_grid)
                .in_set(NDitCoreSet::ProcessInputs),
        )
        .add_systems(
            Update,
            (
                sys_react_to_node_op.in_set(NDitCoreSet::PostProcessCommands),
                (
                    available_moves::sys_adjust_available_moves,
                    range_of_action::get_range_of_action,
                    grid_tooltip::sys_grid_ui_tooltip,
                )
                    .chain()
                    .after(super::node_ui_op::sys_adjust_selected_entity)
                    .in_set(NDitCoreSet::PostProcessUiOps),
                grid_animation::sys_grid_animations.in_set(NDitCoreSet::PostProcessCommands),
            ),
        )
        .add_systems(
            RENDER_TTY_SCHEDULE,
            (scroll::adjust_scroll, render_grid::render_grid_system)
                .chain()
                .in_set(RenderTtySet::PostCalculateLayout),
        );
    }
}

impl NodeUi for GridUi {
    const NAME: &'static str = "Grid UI";
    type UiBundleExtras = (
        Scroll2d,
        MouseEventListener,
        UiFocusOnClick,
        HoverPoint,
        Tooltip,
    );
    type UiPlugin = Self;

    fn initial_style(node_q: &NodeUiQItem) -> StyleTty {
        use taffy::prelude::*;

        StyleTty(taffy::prelude::Style {
            size: Size {
                width: Dimension::Auto,
                height: Dimension::Auto,
            },
            max_size: Size {
                width: Dimension::Points((node_q.grid.width() * 3 + 1) as f32),
                height: Dimension::Points((node_q.grid.height() * 2 + 1) as f32),
            },
            border: Rect {
                left: LengthPercentage::Points(1.0),
                ..TaffyZero::ZERO
            },
            flex_grow: 1.0,
            ..default()
        })
    }

    fn ui_bundle_extras() -> Self::UiBundleExtras {
        (
            Scroll2d::default(),
            MouseEventListener,
            UiFocusOnClick,
            HoverPoint::default(),
            Tooltip::default(),
        )
    }
}

// TODO move to node_ui
fn sys_react_to_node_op(
    ast_action: Res<Assets<Action>>,
    mut ev_op_result: EventReader<OpResult<NodeOp>>,
    mut res_ui_ops: ResMut<UiOps>,
    nodes: Query<
        (
            &EntityGrid,
            AsDerefCopied<CurrentTurn>,
            AsDerefCopied<ActiveCurio>,
        ),
        With<Node>,
    >,
    players_with_node_ui: Query<(), (With<Player>, With<HasNodeUi>)>,
    player_nodes: Query<AsDerefCopied<InNode>, With<Player>>,
    mut player_uis: Query<
        (
            Entity,
            AsDerefCopied<OnTeam>,
            AsDerefCopied<InNode>,
            &mut TelegraphedAction,
        ),
        (With<Player>, With<HasNodeUi>),
    >,
    curio_actions: Query<&Actions, With<Curio>>,
) {
    for op_result in ev_op_result.read() {
        // Reactions to ops from other players in node
        if op_result.result().is_ok() {
            get_assert!(op_result.source(), player_nodes, |node| {
                match op_result.op() {
                    NodeOp::EndTurn => {
                        let (_, current_turn, _) = get_assert!(node, nodes)?;
                        for (id, team, _, _) in player_uis.iter() {
                            if team == current_turn {
                                res_ui_ops.request(id, NodeUiOp::SetCursorHidden(false));
                            }
                        }
                    },
                    NodeOp::TelegraphAction { action_id } => {
                        let (_, _, active_curio) = get_assert!(node, nodes)?;
                        let actions = curio_actions.get(active_curio?).ok()?;
                        let action_handle = actions.iter().find_map(|action_handle| {
                            let action_def = ast_action.get(action_handle)?;
                            (action_def.id() == action_id).then_some(action_handle.clone())
                        });

                        for (_, _, in_node, mut telegraphed_action) in player_uis.iter_mut() {
                            if in_node == node {
                                **telegraphed_action = action_handle.clone();
                            }
                        }
                    },
                    NodeOp::PerformCurioAction { .. } => {
                        for (_, _, in_node, mut telegraphed_action) in player_uis.iter_mut() {
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
        if !players_with_node_ui.contains(op_result.source()) {
            continue;
        }

        // Reactions to own actions
        if let Ok(metadata) = op_result.result() {
            let player = op_result.source();
            match op_result.op() {
                NodeOp::PerformCurioAction { .. } => {
                    res_ui_ops.request(player, NodeUiOp::ChangeFocus(FocusTarget::Grid));
                    res_ui_ops.request(player, NodeUiOp::SetSelectedAction(None));
                },
                NodeOp::MoveActiveCurio { .. } => {
                    // NOTE this will probably fail when an AI takes an action
                    get_assert!(player, player_nodes, |node| {
                        let (grid, _, _) = get_assert!(node, nodes)?;
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
                NodeOp::ReadyToGo => {
                    res_ui_ops.request(player, NodeUiOp::ChangeFocus(FocusTarget::Grid));
                    res_ui_ops.request(player, NodeUiOp::SetSelectedAction(None));
                },
                NodeOp::EndTurn => {
                    res_ui_ops.request(player, NodeUiOp::ChangeFocus(FocusTarget::Grid));
                    res_ui_ops.request(player, NodeUiOp::SetCursorHidden(true));
                },
                _ => {},
            }
        }
    }
}
