mod registry;
mod render_grid;
mod render_menu;
mod render_square;

pub use crate::term::prelude::*;
use crate::term::{layout::CalculatedSizeTty, render::TerminalRendering, TerminalWindow};
use bevy::{core::FrameCount, ecs::query::WorldQuery};
use game_core::{Actions, EntityGrid, NodePiece, Team};
pub use registry::GlyphRegistry;
pub use render_grid::render_grid;
pub use render_square::render_square;

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
    frame_count: Res<FrameCount>,
    glyph_registry: Res<GlyphRegistry>,
    mut render_grid: Query<
        (
            Entity,
            &CalculatedSizeTty,
            &NodeViewScroll,
            Option<&mut TerminalRendering>,
        ),
        With<GridUi>,
    >,
    node_focus: Res<super::NodeFocus>,
) {
    if let Some(node_data) = node_focus.and_then(|node_id| node_data.get(node_id).ok()) {
        // WIP

        for (render_grid_id, size, scroll, rendering_opt) in render_grid.iter_mut() {
            let grid_rendering =
                render_grid::render_grid(size, scroll, &node_data, &node_pieces, &glyph_registry);
            if let Some(mut rendering) = rendering_opt {
                rendering.update(grid_rendering.clone(), frame_count.0);
            } else {
                log::debug!("Adding grid rendering");
                let rendering = TerminalRendering::new(grid_rendering.clone(), frame_count.0);
                commands
                    .get_entity(render_grid_id)
                    .unwrap()
                    .insert(rendering);
            }
        }
    }
}

pub fn render_menu_system(
    mut commands: Commands,
    node_data: Query<RenderNodeData, With<game_core::Node>>,
    node_pieces: Query<NodePieceMenuData>,
    frame_count: Res<FrameCount>,
    mut render_menu: Query<
        (Entity, &CalculatedSizeTty, Option<&mut TerminalRendering>),
        With<RenderMenu>,
    >,
    node_focus: Res<super::NodeFocus>,
) {
    if let Some(node_data) = node_focus.and_then(|node_id| node_data.get(node_id).ok()) {
        for (render_menu_id, size, rendering_opt) in render_menu.iter_mut() {
            let menu_rendering = render_menu::render_menu(&node_data, &node_pieces, size);
            if let Some(mut rendering) = rendering_opt {
                rendering.update(menu_rendering.clone(), frame_count.0);
            } else {
                log::debug!("Adding menu rendering");
                let rendering = TerminalRendering::new(menu_rendering.clone(), frame_count.0);
                commands
                    .get_entity(render_menu_id)
                    .unwrap()
                    .insert(rendering);
            }
        }
    }
}

pub fn render_title_bar_system(
    mut commands: Commands,
    frame_count: Res<FrameCount>,
    mut render_title_bar: Query<(Entity, Option<&mut TerminalRendering>), With<RenderTitleBar>>,
) {
    let rendered_text = vec!["n_dit".to_owned()];
    for (id, rendering_opt) in render_title_bar.iter_mut() {
        if let Some(mut rendering) = rendering_opt {
            rendering.update(rendered_text.clone(), frame_count.0);
        } else {
            log::debug!("Adding title bar rendering");
            let rendering = TerminalRendering::new(rendered_text.clone(), frame_count.0);
            commands.get_entity(id).unwrap().insert(rendering);
        }
    }
}
