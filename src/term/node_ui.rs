mod inputs;
mod setup;
mod titlebar_ui;
mod menu_ui;
mod grid_ui;
mod registry;

use crate::term::{prelude::*, TerminalFocusMode};
use bevy::ecs::query::WorldQuery;
use bevy::reflect::{FromReflect, Reflect};
use game_core::EntityGrid;

use self::menu_ui::{
    MenuUiActions, MenuUiDescription, MenuUiLabel, MenuUiStats, NodeUi,
};

use registry::GlyphRegistry;

use super::render::RenderTtySet;

/// Event that tells us to show a specific Node entity
#[derive(Debug)]
pub struct ShowNode(pub Entity);

/// If there are multiple Nodes, this is the node that is being rendered to the screen
#[derive(Debug, Deref, DerefMut, Resource, Default)]
pub struct NodeFocus(pub Option<Entity>);

/// Plugin for NodeUI
#[derive(Default)]
pub struct NodeUiPlugin;

/// Component that tells the UI which entity the node cursor is over
#[derive(Component, Debug, Deref)]
pub struct SelectedEntity(pub Option<Entity>);

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
    selected_entity: &'static SelectedEntity,
}

impl Plugin for NodeUiPlugin {

    fn build(&self, app: &mut App) {
        app.init_resource::<GlyphRegistry>()
            .init_resource::<NodeFocus>()
            .add_event::<ShowNode>()
            .add_system(setup::create_node_ui.in_schedule(OnEnter(TerminalFocusMode::Node)))
            .add_system(inputs::node_cursor_controls.in_base_set(CoreSet::PreUpdate))
            .add_systems(MenuUiActions::ui_systems())
            .add_systems(MenuUiLabel::ui_systems())
            .add_systems(MenuUiStats::ui_systems())
            .add_systems(MenuUiDescription::ui_systems())
            .add_systems(
                (
                    grid_ui::adjust_scroll.before(grid_ui::render_grid_system),
                    grid_ui::render_grid_system,
                    titlebar_ui::render_title_bar_system,
                )
                    .in_set(OnUpdate(TerminalFocusMode::Node))
                    .in_set(RenderTtySet::PostCalculateLayout),
            );
    }
}