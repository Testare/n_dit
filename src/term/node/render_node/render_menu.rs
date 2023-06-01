use crate::term::layout::CalculatedSizeTty;

use super::RenderNodeDataReadOnlyItem;
use game_core::node::NodePiece;
use game_core::{prelude::*, Team};
use pad::PadStr;

pub fn render_menu(
    node_render_data: &RenderNodeDataReadOnlyItem,
    node_pieces: &Query<(&NodePiece, Option<&Team>)>,
    bounds: &CalculatedSizeTty,
) -> Vec<String> {
    if let Some(selected_entity) = node_render_data
        .grid
        .item_at(**node_render_data.node_cursor)
    {
        log::debug!("bounds: {:?}", bounds);
        let (selected_piece, _) = node_pieces
            .get(selected_entity)
            .expect("entities in entity grid should have NodePiece components");
        vec![selected_piece
            .display_name()
            .clone()
            .with_exact_width(bounds.width() as usize)]
    } else {
        vec![]
    }
    // Get node cursor
    // Get Entity from that position
    // Determine if it is a curio (friendly or not), pickup, or access point
}
