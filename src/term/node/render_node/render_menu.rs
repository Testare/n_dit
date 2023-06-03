use crate::term::layout::CalculatedSizeTty;

use super::RenderNodeDataReadOnlyItem;
use game_core::node::NodePiece;
use game_core::{prelude::*, Actions, Team};
use pad::PadStr;

pub fn render_menu(
    node_render_data: &RenderNodeDataReadOnlyItem,
    node_pieces: &Query<(&NodePiece, Option<&Team>, Option<&Actions>)>,
    bounds: &CalculatedSizeTty,
) -> Vec<String> {
    if let Some(selected_entity) = node_render_data
        .grid
        .item_at(**node_render_data.node_cursor)
    {
        log::debug!("bounds: {:?}", bounds);
        let (selected_piece, team, actions) = node_pieces
            .get(selected_entity)
            .expect("entities in entity grid should have NodePiece components");
        let mut unbound_vec = vec![
            Some(selected_piece.display_name().clone()),
            Some(format!("{0:-^1$}", "-", bounds.width())),
            team.map(|team| format!("Team: {:?}", team)),
            team.map(|_| "".to_owned()),
        ];
        if let Some(actions) = actions {
            for action in actions.iter() {
                unbound_vec.push(Some(action.name.clone()));
            }
        }

        unbound_vec
            .into_iter()
            .filter_map(|str| Some(str?.with_exact_width(bounds.width())))
            .collect()
    } else {
        vec![]
    }
    // Get node cursor
    // Get Entity from that position
    // Determine if it is a curio (friendly or not), pickup, or access point
}
