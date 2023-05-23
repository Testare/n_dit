use game_core::{prelude::*, Team};
use game_core::node::NodePiece;
use super::RenderNodeDataItem;
use unicode_width::UnicodeWidthStr;

pub fn render_menu(
    node_render_data: &RenderNodeDataItem,
    node_pieces: &Query<(&NodePiece, Option<&Team>)>,
    bounds: UVec2,
) -> Vec<String> {
    if let Some(selected_entity) = node_render_data.grid.item_at(**node_render_data.node_cursor) {
        let (selected_piece, _) = node_pieces.get(selected_entity).expect("entities in entity grid should have NodePiece components");
        // TODO fit to bounds
        vec![selected_piece.display_name().clone()]
    } else {
        vec![]
    }
    // Get node cursor
    // Get Entity from that position
    // Determine if it is a curio (friendly or not), pickup, or access point

}