mod registry;
mod render_grid;
mod render_menu;
mod render_square;

use bevy::ecs::query::WorldQuery;

pub use crate::term::prelude::*;
pub use registry::GlyphRegistry;
pub use render_grid::render_grid;
pub use render_square::render_square;

use crate::term::layout::CalculatedSizeTty;
use crate::term::render::UpdateRendering;
use game_core::{EntityGrid, NodePiece, Team};

use self::render_menu::NodePieceMenuData;

use super::NodeCursor;

#[derive(Component)]
pub struct GridUi;

#[derive(Component)]
pub struct RenderMenu;

#[derive(Component)]
pub struct RenderNode;

#[derive(Component)]
pub struct RenderTitleBar;

#[derive(Component, Debug, Default, Deref, DerefMut, FromReflect, Reflect)]
pub struct NodeViewScroll(pub UVec2);

#[derive(WorldQuery)]
#[world_query(mutable)]
pub struct RenderNodeData {
    entity: Entity,
    grid: &'static EntityGrid,
    node_cursor: &'static NodeCursor,
}

pub fn render_grid_system(
    mut commands: Commands,
    node_data: Query<RenderNodeData, With<game_core::Node>>,
    node_pieces: Query<(&NodePiece, Option<&Team>)>,
    glyph_registry: Res<GlyphRegistry>,
    render_grid: Query<
        (
            Entity,
            &CalculatedSizeTty,
            &NodeViewScroll,
        ),
        With<GridUi>,
    >,
    node_focus: Res<super::NodeFocus>,
) {
    if let Some(node_data) = node_focus.and_then(|node_id| node_data.get(node_id).ok()) {
        if let Ok((render_grid_id, size, scroll)) = render_grid.get_single() {
            let grid_rendering =
                render_grid::render_grid(size, scroll, &node_data, &node_pieces, &glyph_registry);
            
            commands
                .get_entity(render_grid_id)
                .unwrap()
                .update_rendering(grid_rendering);
        }
    }
}

pub fn render_menu_system(
    mut commands: Commands,
    node_data: Query<RenderNodeData, With<game_core::Node>>,
    node_pieces: Query<NodePieceMenuData>,
    render_menu: Query<
        (Entity, &CalculatedSizeTty),
        With<RenderMenu>,
    >,
    node_focus: Res<super::NodeFocus>,
) {
    if let Some(node_data) = node_focus.and_then(|node_id| node_data.get(node_id).ok()) {
        if let Ok((render_menu_id, size)) = render_menu.get_single() {
            let menu_rendering = render_menu::render_menu(&node_data, &node_pieces, size);
            commands.get_entity(render_menu_id)
                .unwrap()
                .update_rendering(menu_rendering);
        }
    }
}

pub fn render_title_bar_system(
    mut commands: Commands,
    render_title_bar: Query<Entity, With<RenderTitleBar>>,
) {
    let rendered_text = vec!["n_dit".to_owned()];
    if let Ok(id) = render_title_bar.get_single() {
        commands.get_entity(id)
            .update_rendering(rendered_text.clone());
    }
}
