mod actions;
mod card_selection;
mod description;
mod label;
mod simple_submenu;
mod stats;

pub use actions::MenuUiActions;
use bevy::ecs::query::WorldQuery;
use bevy::ecs::system::SystemParam;
pub use card_selection::MenuUiCardSelection;
use charmi::CharacterMapImage;
pub use description::MenuUiDescription;
use game_core::card::{Actions, Description, MaximumSize, MovementSpeed};
use game_core::node::{AccessPoint, Curio, IsTapped, MovesTaken, NodePiece, Pickup, Team};
use game_core::prelude::*;
pub use label::MenuUiLabel;
pub use stats::MenuUiStats;

use super::{NodeUi, SelectedAction, SelectedEntity};
use crate::layout::{CalculatedSizeTty, StyleTty};

pub trait SimpleSubmenu {
    const NAME: &'static str;
    type RenderSystemParam: SystemParam;
    type UiBundleExtras: Bundle;

    fn initial_style() -> StyleTty {
        use taffy::prelude::*;

        StyleTty(taffy::prelude::Style {
            display: Display::None,
            min_size: Size {
                width: Dimension::Auto,
                height: Dimension::Points(0.0),
            },
            ..default()
        })
    }

    fn height(selected: &NodePieceQItem<'_>) -> Option<usize>;

    fn render(
        player: Entity,
        selected: &NodePieceQItem<'_>,
        size: &CalculatedSizeTty,
        sys_param: &<Self::RenderSystemParam as SystemParam>::Item<'_, '_>,
    ) -> Option<CharacterMapImage>;

    fn ui_bundle_extras() -> Self::UiBundleExtras;
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
