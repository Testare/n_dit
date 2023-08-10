mod button_ui;
mod grid_ui;
mod inputs;
mod menu_ui;
mod messagebar_ui;
mod node_ui_op;
mod registry;
mod setup;
mod titlebar_ui;

use bevy::ecs::query::{ReadOnlyWorldQuery, WorldQuery};
use bevy::reflect::Reflect;
use bevy::utils::HashSet;
use game_core::node::NodeOp;
use game_core::player::ForPlayer;
use game_core::NDitCoreSet;

use self::grid_ui::GridUi;
use self::menu_ui::{
    MenuUiActions, MenuUiCardSelection, MenuUiDescription, MenuUiLabel, MenuUiStats,
};
pub use self::messagebar_ui::MessageBarUi;
pub use self::node_ui_op::NodeUiOp;
use self::registry::GlyphRegistry;
use self::titlebar_ui::TitleBarUi;
use super::layout::StyleTty;
use super::render::TerminalRendering;
use crate::prelude::*;
use crate::TerminalFocusMode;

/// Event that tells us to show a specific Node entity
/// Should likely be replaced with a gamecore Op
#[derive(Debug, Event)]
pub struct ShowNode {
    pub player: Entity,
    pub node: Entity,
}
/// Plugin for NodeUI
#[derive(Default)]
pub struct NodeUiPlugin;

/// Component that tells the UI which entity the node cursor is over
#[derive(Component, Resource, Debug, Deref, DerefMut)]
pub struct SelectedEntity(pub Option<Entity>);

#[derive(Component, Debug, Deref, DerefMut)]
pub struct SelectedAction(Option<usize>);

#[derive(Component, Debug, Default, Deref, DerefMut)]
pub struct AvailableMoves(HashSet<UVec2>);

#[derive(Component, Debug, Default, Deref, DerefMut)]
pub struct AvailableActionTargets(HashSet<UVec2>);

/// Cursor that the user controls to select pieces in the node
#[derive(Component, Debug, Default, Deref, DerefMut, Reflect)]
pub struct NodeCursor(pub UVec2);

impl Plugin for NodeUiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GlyphRegistry>()
            .add_event::<ShowNode>()
            .add_event::<Op<NodeUiOp>>()
            .add_systems(OnEnter(TerminalFocusMode::Node), setup::create_node_ui)
            .add_systems(
                PreUpdate,
                (
                    inputs::kb_ready,
                    inputs::kb_skirm_focus,
                    button_ui::mouse_ready_button,
                )
                    .in_set(NDitCoreSet::ProcessInputs),
            )
            .add_systems(
                Update,
                (
                    (
                        node_ui_op::sys_node_ui_op_change_focus,
                        node_ui_op::sys_node_ui_op_set_selected_action,
                        node_ui_op::sys_node_ui_op_move_cursor,
                    )
                        .in_set(NDitCoreSet::ProcessUiOps),
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

impl SelectedEntity {
    pub fn of<'a, 'w, 's, Q: WorldQuery, R: ReadOnlyWorldQuery>(
        &self,
        query: &'a Query<'w, 's, Q, R>,
    ) -> Option<<<Q as WorldQuery>::ReadOnly as WorldQuery>::Item<'a>> {
        query.get(self.0?).ok()
    }

    pub fn of_mut<'a, 'w, 's, Q: WorldQuery, R: ReadOnlyWorldQuery>(
        &self,
        query: &'a mut Query<'w, 's, Q, R>,
    ) -> Option<<Q as WorldQuery>::Item<'a>> {
        query.get_mut(self.0?).ok()
    }
}

#[derive(WorldQuery)]
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
