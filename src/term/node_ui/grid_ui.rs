mod available_moves;
mod borders;
mod range_of_action;
mod render_grid;
mod render_square;
mod scroll;

use bevy::ecs::query::WorldQuery;
use game_core::card::MovementSpeed;
use game_core::node::{AccessPoint, InNode, IsTapped, NodePiece, Team};
pub use scroll::Scroll2D;

use super::{
    AvailableActionTargets, AvailableMoves, NodeCursor, NodeUi, SelectedAction, SelectedEntity,
};
use crate::term::prelude::*;
use crate::term::render::RenderTtySet;

#[derive(Component, Default)]
pub struct GridUi;

#[derive(WorldQuery)]
pub struct NodePieceQ {
    piece: &'static NodePiece,
    team: Option<&'static Team>,
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
            (
                available_moves::adjust_available_moves,
                range_of_action::get_range_of_action,
            )
                .chain()
                .in_set(RenderTtySet::PreCalculateLayout),
        )
        .add_systems(
            (scroll::adjust_scroll, render_grid::render_grid_system)
                .chain()
                .in_set(RenderTtySet::PostCalculateLayout),
        );
    }
}

impl NodeUi for GridUi {
    type UiBundle = ();
    type UiPlugin = Self;

    fn ui_bundle() -> Self::UiBundle {
        ()
    }
}
