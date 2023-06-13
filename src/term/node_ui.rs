mod grid_ui;
mod inputs;
mod menu_ui;
mod registry;
mod setup;
mod titlebar_ui;
mod messagebar_ui;

use bevy::ecs::query::WorldQuery;
use bevy::ecs::system::SystemParam;
use bevy::reflect::{FromReflect, Reflect};
use bevy::utils::HashSet;
use game_core::NDitCoreSet;
use game_core::node::Node;
use registry::GlyphRegistry;

use self::menu_ui::{
    MenuUiActions, MenuUiCardSelection, MenuUiDescription, MenuUiLabel, MenuUiStats, NodeUi,
};
use super::render::RenderTtySet;
use crate::term::prelude::*;
use crate::term::TerminalFocusMode;

pub use messagebar_ui::MessageBarUi;

/// Event that tells us to show a specific Node entity
/// Should likely be replaced with a gamecore Op
#[derive(Debug)]
pub struct ShowNode {
    pub pn: usize,
    pub node: Entity,
}

/// If there are multiple Nodes, this is the node that is being rendered to the screen
#[derive(Debug, Deref, DerefMut, Resource, Default)]
pub struct NodeFocus(pub Option<Entity>);

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

/// Cursor that the user controls to select pieces in the node
#[derive(Component, Debug, Default, Deref, DerefMut, FromReflect, Reflect)]
pub struct NodeCursor(pub UVec2);

/// Query for getting common node data in systems
#[derive(WorldQuery)]
#[world_query(mutable)]
pub struct NodeUiQ {
    entity: Entity,
    grid: &'static EntityGrid,
    node_cursor: &'static NodeCursor,
    available_moves: &'static AvailableMoves,
    selected_entity: &'static SelectedEntity,
}

#[derive(SystemParam)]
pub struct NodeUiDataParam<'w, 's> {
    query: Query<'w, 's, NodeUiQ, With<Node>>,
    node_focus: Res<'w, NodeFocus>,
}

impl<'w, 's> NodeUiDataParam<'w, 's> {
    fn node_data(&self) -> Option<NodeUiQReadOnlyItem> {
        self.query.get((**self.node_focus)?).ok()
    }
}

impl Plugin for NodeUiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GlyphRegistry>()
            .init_resource::<NodeFocus>()
            .add_event::<ShowNode>()
            .add_system(setup::create_node_ui.in_schedule(OnEnter(TerminalFocusMode::Node)))
            .add_system(inputs::node_cursor_controls.in_base_set(CoreSet::PreUpdate))
            .add_systems(
                (
                    menu_ui::MenuUiCardSelection::<0>::handle_layout_events,
                    menu_ui::MenuUiActions::handle_layout_events,
                    messagebar_ui::style_message_bar,
                )
                    .in_set(OnUpdate(TerminalFocusMode::Node))
                    .before(NDitCoreSet::ProcessCommands),
            )
            .add_systems(
                (
                    grid_ui::adjust_available_moves,
                    // menu_ui::MenuUiCardSelection::<0>::handle_layout_events,
                )
                    .in_set(OnUpdate(TerminalFocusMode::Node))
                    .in_set(RenderTtySet::PreCalculateLayout),
            )
            .add_systems(MenuUiLabel::ui_systems())
            .add_systems(MenuUiCardSelection::<0>::ui_systems())
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

