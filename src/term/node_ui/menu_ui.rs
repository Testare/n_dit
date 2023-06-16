mod actions;
mod card_selection;
mod description;
mod label;
mod simple_submenu;
mod stats;

pub use actions::MenuUiActions;
use bevy::app::SystemAppConfig;
use bevy::ecs::query::WorldQuery;
use bevy::ecs::system::SystemParam;
pub use card_selection::MenuUiCardSelection;
pub use description::MenuUiDescription;
use game_core::card::{Actions, Description, MaximumSize, MovementSpeed};
use game_core::node::{AccessPoint, Curio, IsTapped, MovesTaken, NodePiece, Pickup, Team};
use game_core::prelude::*;
pub use label::MenuUiLabel;
pub use stats::MenuUiStats;

use super::{NodeUi, SelectedAction, SelectedEntity};
use crate::term::layout::CalculatedSizeTty;

trait SimpleSubmenu {
    type RenderSystemParam: SystemParam;

    fn layout_event_system() -> Option<SystemAppConfig> {
        None
    }

    fn height(selected: &NodePieceQItem<'_>) -> Option<usize>;
    fn render<'w, 's>(
        player: Entity,
        selected: &NodePieceQItem<'_>,
        size: &CalculatedSizeTty,
        sys_param: &<Self::RenderSystemParam as SystemParam>::Item<'w, 's>,
    ) -> Option<Vec<String>>;
}

#[derive(WorldQuery)]
pub struct NodePieceQ {
    entity: Entity,
    piece: &'static NodePiece,
    team: Option<&'static Team>,
    curio: Option<&'static Curio>,
    pickup: Option<&'static Pickup>,
    actions: Option<&'static Actions>,
    description: Option<&'static Description>,
    speed: Option<&'static MovementSpeed>,
    max_size: Option<&'static MaximumSize>,
    moves_taken: Option<&'static MovesTaken>,
    is_tapped: Option<&'static IsTapped>,
    access_point: Option<&'static AccessPoint>,
}
