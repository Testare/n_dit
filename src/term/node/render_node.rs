mod registry;
mod render_grid;
mod render_square;
mod render_menu;

pub use crate::term::prelude::*;
use crate::term::{render::TerminalRendering, TerminalWindow};
use bevy::{core::FrameCount, ecs::query::WorldQuery};
use game_core::{EntityGrid, NodePiece, Team};
use itertools::Itertools;
pub use registry::GlyphRegistry;
pub use render_grid::render_grid;
pub use render_square::render_square;
use unicode_width::UnicodeWidthStr;

use super::NodeCursor;



#[derive(WorldQuery)]
#[world_query(mutable)]
pub struct RenderNodeData {
    entity: Entity,
    grid: &'static EntityGrid,
    node_cursor: &'static NodeCursor,
}

pub fn render_node(
    mut commands: Commands,
    window: Res<TerminalWindow>,
    mut node_grids: Query<
        (RenderNodeData, Option<&mut TerminalRendering>),
        With<game_core::Node>,
    >,
    node_pieces: Query<(&NodePiece, Option<&Team>)>,
    frame_count: Res<FrameCount>,
    glyph_registry: Res<GlyphRegistry>,
) {
    if let Some((node_data, rendering_opt)) = node_grids.iter_mut().next() {

        let menu_width = 12;
        let grid_width = window.width() - menu_width;
        let grid_rendering =
            render_grid::render_grid(window, &node_data, &node_pieces, &glyph_registry);
        let menu_rendering = render_menu::render_menu(&node_data, &node_pieces, UVec2 { x: 12, y: 5});
        let merged_rendering: Vec<String> = menu_rendering.iter().zip_longest(grid_rendering.iter()).map(|rendering_lines| {
            // let (menu_row, griw_row) = rendering_lines.map_any(String::as_str, String::as_str).or("", "");
            let menu_row = rendering_lines.as_ref().left().map(|row|(*row).as_str()).unwrap_or("");
            let grid_row = rendering_lines.as_ref().right().map(|row|(*row).as_str()).unwrap_or("");
            let border = '\\';

            let row_width: usize = UnicodeWidthStr::width(grid_row);
            let padding_size: usize = if row_width < grid_width{
                1 + grid_width - row_width
            } else {
                1
            };
            let menu_row_width: usize = UnicodeWidthStr::width(menu_row);
            let menu_padding_size: usize = if menu_row_width < menu_width {
                menu_width - menu_row_width
            } else {
                0
            }; // TODO logic to truncate if menu_row is greater than menu size...

            format!(
                "{0}{1}{space:menu_padding$.menu_padding$}{0} {2}{space:padding$}{0}\n",
                border,
                menu_row,
                grid_row,
                space = " ",
                menu_padding = menu_padding_size,
                padding = padding_size
            )

        }).collect();

        if let Some(mut rendering) = rendering_opt {
            rendering.update(merged_rendering, frame_count.0);
        } else {
            let rendering = TerminalRendering::new(merged_rendering, frame_count.0);
            commands.get_entity(node_data.entity).unwrap().insert(rendering);
        }
    }
}
