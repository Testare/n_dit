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
use game_core::op::OpResult;
use game_core::player::{ForPlayer, Player};
use game_core::NDitCoreSet;
pub use grid_animation::GridUiAnimation;
pub use scroll::Scroll2D;

use super::menu_ui::MenuUiActions;
use super::{
    AvailableActionTargets, AvailableMoves, NodeCursor, NodeUi, NodeUiQItem, SelectedAction,
    SelectedEntity,
};
use crate::layout::{LayoutMouseTarget, StyleTty, UiFocusNext, UiFocusOnClick};
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
                (
                    sys_react_to_node_op,
                    available_moves::sys_adjust_available_moves,
                    range_of_action::get_range_of_action,
                )
                    .chain()
                    .in_set(NDitCoreSet::PostProcessCommands),
                grid_animation::sys_grid_animations.in_set(NDitCoreSet::PostProcessCommands),
                (
                    grid_animation::sys_update_animations,
                    grid_animation::sys_render_animations,
                    grid_animation::sys_reset_state_after_animation_plays,
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

fn sys_react_to_node_op(
    mut ev_op_result: EventReader<OpResult<NodeOp>>,
    nodes: Query<(&EntityGrid,), With<Node>>,
    action_menus: Query<(Entity, &ForPlayer), With<MenuUiActions>>,
    grid_uis: Query<(Entity, &ForPlayer), With<GridUi>>,
    mut players: Query<
        (
            &InNode,
            &mut NodeCursor,
            &mut SelectedEntity,
            &mut SelectedAction,
            &mut UiFocusNext,
        ),
        With<Player>,
    >,
) {
    for op_result in ev_op_result.iter() {
        if let Ok(metadata) = op_result.result() {
            match op_result.source().op() {
                NodeOp::PerformCurioAction { .. } => {
                    let player = op_result.source().player();
                    players
                        .get_mut(player)
                        .ok()
                        .and_then(|(_, _, _, _, mut focus_next)| {
                            let grid_ui_id = grid_uis
                                .iter()
                                .filter(|(_, for_player)| ***for_player == player)
                                .next()?
                                .0;
                            **focus_next = Some(grid_ui_id);
                            Some(())
                        });
                },
                NodeOp::MoveActiveCurio { .. } => {
                    let player = op_result.source().player();
                    // NOTE this will probably fail when an AI takes an action
                    get_assert_mut!(
                        op_result.source().player(),
                        players,
                        |(
                            node,
                            mut node_cursor,
                            selected_entity,
                            selected_action,
                            mut ui_focus_next,
                        )| {
                            let (grid,) = get_assert!(**node, nodes)?;
                            let curio = metadata.get_required(node::key::CURIO).ok()?;
                            let remaining_moves =
                                metadata.get_required(node::key::REMAINING_MOVES).ok()?;
                            let tapped = metadata.get_or_default(node::key::TAPPED).ok()?;
                            // let active_curio = (**active_curio)?;
                            node_cursor.adjust_to(
                                grid.head(curio)?,
                                selected_entity,
                                selected_action,
                                grid,
                            );
                            if remaining_moves == 0 && !tapped {
                                let action_menu_id = action_menus
                                    .iter()
                                    .filter(|(_, for_player)| ***for_player == player)
                                    .next()?
                                    .0;
                                **ui_focus_next = Some(action_menu_id);
                            }
                            Some(())
                        }
                    );
                },
                _ => {},
            }
        }
    }
}
