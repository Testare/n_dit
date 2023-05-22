mod registry;
mod render_grid;
mod render_square;

pub use crate::term::prelude::*;
use crate::term::{render::TerminalRendering, TerminalWindow};
use bevy::core::FrameCount;
use game_core::{EntityGrid, NodePiece, Team};
pub use registry::GlyphRegistry;
pub use render_grid::render_grid;
pub use render_square::render_square;

use super::NodeCursor;

pub fn render_node(
    mut commands: Commands,
    window: Res<TerminalWindow>,
    mut node_grids: Query<
        (
            Entity,
            &EntityGrid,
            &NodeCursor,
            Option<&mut TerminalRendering>,
        ),
        With<game_core::Node>,
    >,
    node_pieces: Query<(&NodePiece, Option<&Team>)>,
    frame_count: Res<FrameCount>,
    glyph_registry: Res<GlyphRegistry>,
) {
    if let Some((entity, grid, node_cursor, rendering_opt)) = node_grids.iter_mut().next() {
        let grid_rendering =
            render_grid::render_grid(window, grid, node_cursor, node_pieces, &glyph_registry);
        if let Some(mut rendering) = rendering_opt {
            rendering.update(grid_rendering, frame_count.0);
        } else {
            let rendering = TerminalRendering::new(grid_rendering, frame_count.0);
            commands.get_entity(entity).unwrap().insert(rendering);
        }
    }
}
