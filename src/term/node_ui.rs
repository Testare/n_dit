mod grid_ui;
mod inputs;
mod menu_ui;
mod messagebar_ui;
mod registry;
mod setup;
mod titlebar_ui;

use bevy::ecs::query::{ReadOnlyWorldQuery, WorldQuery};
use bevy::reflect::{FromReflect, Reflect};
use bevy::utils::HashSet;
use game_core::NDitCoreSet;
pub use messagebar_ui::MessageBarUi;
use registry::GlyphRegistry;

use self::menu_ui::{
    MenuUiActions, MenuUiCardSelection, MenuUiDescription, MenuUiLabel, MenuUiStats, NodeUi,
};
use super::render::RenderTtySet;
use crate::term::prelude::*;
use crate::term::TerminalFocusMode;

/// Event that tells us to show a specific Node entity
/// Should likely be replaced with a gamecore Op
#[derive(Debug)]
pub struct ShowNode {
    pub player: Entity,
    pub node: Entity,
}

/// Plugin for NodeUI
#[derive(Default)]
pub struct NodeUiPlugin;

/// Component that tells the UI which entity the node cursor is over
#[derive(Component, Resource, Debug, Deref)]
pub struct SelectedEntity(pub Option<Entity>);

#[derive(Component, Debug, Deref, DerefMut)]
pub struct SelectedAction(Option<usize>);

#[derive(Component, Debug, Default, Deref, DerefMut)]
pub struct AvailableMoves(HashSet<UVec2>);

#[derive(Component, Debug, Default, Deref, DerefMut)]
pub struct AvailableActionTargets(HashSet<UVec2>);

/// Cursor that the user controls to select pieces in the node
#[derive(Component, Debug, Default, Deref, DerefMut, FromReflect, Reflect)]
pub struct NodeCursor(pub UVec2);

impl Plugin for NodeUiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GlyphRegistry>()
            .add_event::<ShowNode>()
            .add_system(setup::create_node_ui.in_schedule(OnEnter(TerminalFocusMode::Node)))
            .add_system(inputs::grid_ui_keyboard_controls.in_base_set(CoreSet::PreUpdate))
            .add_systems(
                (
                    menu_ui::MenuUiCardSelection::handle_layout_events,
                    menu_ui::MenuUiActions::handle_layout_events,
                    messagebar_ui::style_message_bar,
                )
                    .in_set(OnUpdate(TerminalFocusMode::Node))
                    .before(NDitCoreSet::ProcessCommands),
            )
            .add_systems(
                (
                    grid_ui::adjust_available_moves,
                    grid_ui::get_range_of_action,
                )
                    .chain()
                    .in_set(OnUpdate(TerminalFocusMode::Node))
                    .in_set(RenderTtySet::PreCalculateLayout),
            )
            .add_systems(MenuUiLabel::ui_systems())
            .add_systems(MenuUiCardSelection::ui_systems())
            .add_systems(MenuUiActions::ui_systems())
            .add_systems(MenuUiStats::ui_systems())
            .add_systems(MenuUiDescription::ui_systems())
            .add_systems(
                (
                    grid_ui::adjust_scroll.before(grid_ui::render_grid_system),
                    grid_ui::render_grid_system,
                    titlebar_ui::render_title_bar_system,
                    messagebar_ui::render_message_bar,
                )
                    .in_set(OnUpdate(TerminalFocusMode::Node))
                    .in_set(RenderTtySet::PostCalculateLayout),
            );
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
