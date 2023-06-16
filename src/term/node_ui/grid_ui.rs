mod borders;
mod render_square;
mod render_grid;
mod available_moves;
mod range_of_action;
mod scroll;

use bevy::ecs::query::WorldQuery;
use game_core::card::{MovementSpeed};
use game_core::node::{AccessPoint, InNode, IsTapped, NodePiece, Team};

use super::{AvailableActionTargets, AvailableMoves, NodeCursor, SelectedAction, SelectedEntity};
use crate::term::prelude::*;

pub use render_grid::render_grid_system;
pub use available_moves::adjust_available_moves;
pub use range_of_action::get_range_of_action;
pub use scroll::adjust_scroll;
pub use scroll::Scroll2D;

#[derive(Component)]
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