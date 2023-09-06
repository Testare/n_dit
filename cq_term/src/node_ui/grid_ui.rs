mod available_moves;
mod borders;
mod grid_animation;
mod grid_inputs;
mod range_of_action;
mod render_grid;
mod render_square;
mod scroll;

use bevy::ecs::query::WorldQuery;
use game_core::card::MovementSpeed;
use game_core::node::{self, AccessPoint, InNode, IsTapped, Node, NodeOp, NodePiece};
use game_core::op::{OpResult, OpSubtype};
use game_core::player::Player;
use game_core::NDitCoreSet;
pub use grid_animation::GridUiAnimation;
pub use scroll::Scroll2D;

use super::node_ui_op::FocusTarget;
use super::{
    AvailableActionTargets, AvailableMoves, HasNodeUi, NodeCursor, NodeUi, NodeUiOp, NodeUiQItem,
    SelectedAction, SelectedEntity,
};
use crate::layout::{LayoutMouseTarget, StyleTty, UiFocusOnClick};
use crate::prelude::*;
use crate::render::{RenderTtySet, RENDER_TTY_SCHEDULE};
use crate::TerminalFocusMode;

#[derive(Component, Default)]
pub struct GridUi;

#[derive(WorldQuery)]
pub struct NodePieceQ {
    piece: &'static NodePiece,
    speed: Option<&'static MovementSpeed>,
    is_tapped: Option<&'static IsTapped>,
    access_point: Option<&'static AccessPoint>,
}

#[derive(WorldQuery)]
pub struct PlayerUiQ {
    entity: Entity,
    selected_entity: &'static SelectedEntity,
    selected_action: &'static SelectedAction,
    node_cursor: &'static NodeCursor,
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
            OnEnter(TerminalFocusMode::Node),
            grid_animation::sys_create_grid_animation_player,
        )
        .add_systems(
            Update,
            (
                sys_react_to_node_op.in_set(NDitCoreSet::PostProcessCommands),
                (
                    available_moves::sys_adjust_available_moves,
                    range_of_action::get_range_of_action,
                )
                    .chain()
                    .after(super::node_ui_op::sys_adjust_selected_entity)
                    .in_set(NDitCoreSet::PostProcessUiOps),
                grid_animation::sys_grid_animations.in_set(NDitCoreSet::PostProcessCommands),
                (
                    grid_animation::sys_update_animations,
                    grid_animation::sys_render_animations,
                )
                    .chain()
                    .before(NDitCoreSet::PostProcessCommands),
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
    type UiBundleExtras = (Scroll2D, LayoutMouseTarget, UiFocusOnClick);
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
                left: Dimension::Points(1.0),
                ..default()
            },
            flex_grow: 1.0,
            ..default()
        })
    }

    fn ui_bundle_extras() -> Self::UiBundleExtras {
        (Scroll2D::default(), LayoutMouseTarget, UiFocusOnClick)
    }
}

// TODO move to node_ui
fn sys_react_to_node_op(
    mut ev_op_result: EventReader<OpResult<NodeOp>>,
    nodes: Query<(&EntityGrid,), With<Node>>,
    players: Query<(&InNode,), (With<Player>, With<HasNodeUi>)>,
    mut ev_node_ui_op: EventWriter<Op<NodeUiOp>>,
) {
    for op_result in ev_op_result.iter() {
        if !players.contains(op_result.source().player()) {
            continue;
        }
        if let Ok(metadata) = op_result.result() {
            match op_result.source().op() {
                NodeOp::PerformCurioAction { .. } => {
                    let player = op_result.source().player();
                    NodeUiOp::ChangeFocus(FocusTarget::Grid)
                        .for_p(player)
                        .send(&mut ev_node_ui_op);
                    NodeUiOp::SetSelectedAction(None)
                        .for_p(player)
                        .send(&mut ev_node_ui_op)
                },
                NodeOp::MoveActiveCurio { .. } => {
                    let player = op_result.source().player();
                    // NOTE this will probably fail when an AI takes an action
                    get_assert!(player, players, |(node,)| {
                        let (grid,) = get_assert!(**node, nodes)?;
                        let curio = metadata.get_required(node::key::CURIO).ok()?;
                        let remaining_moves =
                            metadata.get_required(node::key::REMAINING_MOVES).ok()?;
                        let tapped = metadata.get_or_default(node::key::TAPPED).ok()?;
                        NodeUiOp::MoveNodeCursor(grid.head(curio)?.into())
                            .for_p(player)
                            .send(&mut ev_node_ui_op);
                        if remaining_moves == 0 && !tapped {
                            NodeUiOp::ChangeFocus(FocusTarget::ActionMenu)
                                .for_p(player)
                                .send(&mut ev_node_ui_op);
                        }
                        Some(())
                    });
                },
                NodeOp::ReadyToGo => {
                    NodeUiOp::ChangeFocus(FocusTarget::Grid)
                        .for_p(op_result.source().player())
                        .send(&mut ev_node_ui_op);
                },
                _ => {},
            }
        }
    }
}
