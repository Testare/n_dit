mod borders;
mod calculate_ui_components;
mod grid_animation;
mod grid_inputs;
mod grid_tooltip;
mod render_grid;
mod render_square;
mod scroll;

use bevy::ecs::query::{Has, QueryData};
use game_core::card::{Actions, MovementSpeed};
use game_core::node::{
    AccessPoint, Curio, InNode, IsTapped, MovesTaken, Node, NodePiece, OnTeam, Pickup,
};
use game_core::player::{ForPlayer, Player};
use game_core::NDitCoreSet;
pub use grid_animation::GridUiAnimation;

use self::grid_inputs::GridContextActions;
use super::{
    AvailableActionTargets, AvailableMoves, CursorIsHidden, NodeCursor, NodeUi, NodeUiQItem,
    SelectedAction, SelectedNodePiece, TelegraphedAction,
};
use crate::base_ui::{HoverPoint, Scroll2d, Tooltip};
use crate::input_event::MouseEventListener;
use crate::layout::{StyleTty, UiFocusOnClick};
use crate::prelude::*;
use crate::render::{RenderTtySet, RENDER_TTY_SCHEDULE};

#[derive(Component, Default)]
pub struct GridUi;

impl Plugin for GridUi {
    fn build(&self, app: &mut App) {
        app.init_resource::<GridContextActions>()
            .add_systems(
                PreUpdate,
                (grid_inputs::handle_layout_events, grid_inputs::kb_grid)
                    .in_set(NDitCoreSet::ProcessInputs),
            )
            .add_systems(
                Update,
                (
                    grid_animation::sys_grid_animations.in_set(NDitCoreSet::PostProcessCommands),
                    calculate_ui_components::sys_hover_grid_point
                        .in_set(NDitCoreSet::PostProcessUiOps),
                    grid_inputs::sys_grid_context_actions
                        .after(calculate_ui_components::sys_hover_grid_point),
                    (
                        calculate_ui_components::sys_adjust_available_moves,
                        (
                            calculate_ui_components::sys_path_under_hover
                                .after(calculate_ui_components::sys_hover_grid_point),
                            (
                                calculate_ui_components::sys_get_range_of_action,
                                grid_tooltip::sys_grid_ui_tooltip,
                            )
                                .chain(),
                        ),
                    )
                        .chain()
                        .after(super::node_ui_op::sys_adjust_selected_entity)
                        .in_set(NDitCoreSet::PostProcessUiOps),
                ),
            )
            .add_systems(
                RENDER_TTY_SCHEDULE,
                (
                    (scroll::adjust_scroll, render_grid::render_grid_system)
                        .chain()
                        .in_set(RenderTtySet::PostCalculateLayout),
                    sys_react_to_changed_node.in_set(RenderTtySet::PreCalculateLayout),
                ),
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
        GridHoverPoint,
        LastGridHoverPoint,
        PathToGridPoint,
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
            GridHoverPoint::default(),
            LastGridHoverPoint::default(),
            PathToGridPoint::default(),
            Tooltip::default(),
        )
    }
}

fn sys_react_to_changed_node(
    q_player_changed: Query<(Entity, &InNode), (Changed<InNode>, With<Player>)>,
    mut q_grid_ui: Query<(&ForPlayer, AsDerefMut<StyleTty>), With<GridUi>>,
    q_node: Query<&EntityGrid, With<Node>>,
) {
    for (player_id, &InNode(node_id)) in q_player_changed.iter() {
        for (&ForPlayer(for_player), mut style) in q_grid_ui.iter_mut() {
            if player_id != for_player {
                continue;
            }
            get_assert!(node_id, q_node, |grid| {
                use taffy::prelude::*;
                style.max_size = Size {
                    width: Dimension::Points((grid.width() * 3 + 1) as f32),
                    height: Dimension::Points((grid.height() * 2 + 1) as f32),
                };
                Some(())
            });
        }
    }
}

#[derive(Component, Default, Deref, DerefMut)]
pub struct GridHoverPoint(Option<UVec2>);

#[derive(Component, Default, Deref, DerefMut)]
pub struct LastGridHoverPoint(UVec2);

#[derive(Component, Default, Deref, DerefMut)]
pub struct PathToGridPoint(Vec<(UVec2, Compass)>);

#[derive(QueryData)]
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

#[derive(QueryData)]
pub struct PlayerUiQ {
    entity: Entity,
    selected_entity: &'static SelectedNodePiece,
    selected_action: &'static SelectedAction,
    telegraphed_action: &'static TelegraphedAction,
    node_cursor: &'static NodeCursor,
    cursor_is_hidden: AsDerefCopied<CursorIsHidden>,
    available_moves: &'static AvailableMoves,
    available_action_targets: &'static AvailableActionTargets,
    in_node: &'static InNode,
}
