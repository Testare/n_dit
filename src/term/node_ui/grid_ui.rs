mod available_moves;
mod borders;
mod grid_inputs;
mod range_of_action;
mod render_grid;
mod render_square;
mod scroll;

use bevy::ecs::query::WorldQuery;
use game_core::card::MovementSpeed;
use game_core::node::{AccessPoint, ActiveCurio, InNode, IsTapped, Node, NodeOp, NodePiece};
use game_core::player::Player;
use game_core::{NDitCoreSet, OpResult};
pub use scroll::Scroll2D;

use super::{
    AvailableActionTargets, AvailableMoves, NodeCursor, NodeUi, NodeUiQItem, SelectedAction,
    SelectedEntity,
};
use crate::term::layout::{LayoutMouseTarget, StyleTty, UiFocusOnClick};
use crate::term::prelude::*;
use crate::term::render::{RenderTtySet, RENDER_TTY_SCHEDULE};

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
            Update,
            (
                adjust_node_cursor_when_curio_moves,
                available_moves::adjust_available_moves,
                range_of_action::get_range_of_action,
            )
                .chain()
                .in_set(NDitCoreSet::PostProcessCommands),
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

fn adjust_node_cursor_when_curio_moves(
    mut ev_op_result: EventReader<OpResult<NodeOp>>,
    nodes: Query<(&EntityGrid, &ActiveCurio), With<Node>>,
    mut players: Query<
        (
            &InNode,
            &mut NodeCursor,
            &mut SelectedEntity,
            &mut SelectedAction,
        ),
        With<Player>,
    >,
) {
    for op_result in ev_op_result.iter() {
        if op_result.result().is_err() {
            continue;
        }
        match op_result.source().op() {
            NodeOp::MoveActiveCurio { .. } => {
                get_assert_mut!(op_result.source().player(), players, |(
                    node,
                    mut node_cursor,
                    selected_entity,
                    selected_action,
                )| {
                    let (grid, active_curio) = get_assert!(**node, nodes)?;
                    let active_curio = (**active_curio)?;
                    node_cursor.adjust_to(
                        grid.head(active_curio)?,
                        selected_entity,
                        selected_action,
                        grid,
                    );
                    Some(())
                });
            },
            NodeOp::PerformCurioAction { .. } => {
                get_assert_mut!(op_result.source().player(), players, |(
                    node,
                    mut node_cursor,
                    selected_entity,
                    selected_action,
                )| {
                    let (grid, _) = get_assert!(**node, nodes)?;
                    let cursor_pt = **node_cursor;
                    node_cursor.adjust_to(cursor_pt, selected_entity, selected_action, grid);
                    Some(())
                });
            },
            _ => {},
        }
    }
}
