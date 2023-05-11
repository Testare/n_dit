mod render_node;

use game_core::{self, EntityGrid, NodePiece, Team};
use super::TerminalWindow;
use game_core::prelude::*;
use render_node::GlyphRegistry;

pub struct RenderPlugin;

impl Plugin for RenderPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GlyphRegistry>()
            .add_system(render_node);
    }

}

pub fn render_node(
    windows: Query<&TerminalWindow>,
    node_grids: Query<&EntityGrid, With<game_core::Node>>,
    node_pieces: Query<(&NodePiece, Option<&Team>)>,
    glyph_registry: Res<GlyphRegistry>,
) {
    render_node::render_grid(windows, node_grids, node_pieces, &glyph_registry);
}