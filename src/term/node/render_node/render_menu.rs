use crate::term::layout::CalculatedSizeTty;

use super::RenderNodeDataReadOnlyItem;
use bevy::ecs::query::WorldQuery;
use game_core::node::NodePiece;
use game_core::{prelude::*, Actions, Curio, Description, MaximumSize, Mon, MovementSpeed, Team};
use pad::PadStr;

#[derive(WorldQuery)]
pub struct NodePieceMenuData {
    piece: &'static NodePiece,
    team: Option<&'static Team>,
    curio: Option<&'static Curio>,
    mon: Option<&'static Mon>,
    actions: Option<&'static Actions>,
    description: Option<&'static Description>,
    speed: Option<&'static MovementSpeed>,
    max_size: Option<&'static MaximumSize>,
}

pub fn render_menu(
    node_render_data: &RenderNodeDataReadOnlyItem,
    node_pieces: &Query<NodePieceMenuData>,
    size: &CalculatedSizeTty,
) -> Vec<String> {
    if let Some(selected_entity) = node_render_data
        .grid
        .item_at(**node_render_data.node_cursor)
    {
        let selected = node_pieces
            .get(selected_entity)
            .expect("entities in entity grid should have NodePiece components");

        let mut unbound_vec = vec![
            selected.piece.display_id().clone(),
            // selected.team.map(|team| format!("Team: {:?}", team)),
            // selected.team.map(|_| "".to_owned()),
        ];

        if let Some(name) = selected
            .curio
            .map(Curio::name)
            .or_else(|| selected.mon.and(Some("Mon")))
            .map(str::to_owned)
        {
            unbound_vec.push(name);
        }

        if selected.max_size.is_some() || selected.speed.is_some() {
            unbound_vec.push(format!("{0:-<1$}", "-Stats", size.width()));
            if let Some(max_size) = selected.max_size {
                let size = node_render_data.grid.len_of(selected_entity);
                unbound_vec.push(format!("Size:  {}/{}", size, **max_size));
            }
            if let Some(speed) = selected.speed {
                unbound_vec.push(format!("Speed: {}", **speed));
            }
        }
        if let Some(actions) = selected.actions {
            unbound_vec.push(format!("{0:-<1$}", "-Actions", size.width()));
            for action in actions.iter() {
                // Record position of action
                unbound_vec.push(action.name.clone());
            }
        }
        if let Some(description) = selected.description {
            unbound_vec.push(format!("{0:-<1$}", "-Desc", size.width()));
            let wrapped_desc = textwrap::wrap(description.as_str(), size.width());
            for desc_line in wrapped_desc.into_iter() {
                unbound_vec.push(desc_line.into_owned());
            }
        }
        unbound_vec
            .into_iter()
            .map(|line| (line.with_exact_width(size.width())))
            .take(size.height())
            .collect()
    } else {
        vec![]
    }
    // Get node cursor
    // Get Entity from that position
    // Determine if it is a curio (friendly or not), pickup, or access point
}
