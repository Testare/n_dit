use super::registry::GlyphRegistry;
use crate::term::configuration::DrawConfiguration;
use game_core::prelude::*;
use game_core::{NodePiece, Team};

const UNKNOWN_NODE_PIECE: &'static str = "??";
const FILL_GLYPH: &'static str = "[]";

pub fn render_square(
    position: usize,
    entity: Entity,
    node_pieces: &Query<(&NodePiece, Option<&Team>)>,
    node_piece_render_registry: &GlyphRegistry,
    configuration: &DrawConfiguration,
) -> String {
    let (node_piece, team_opt) = node_pieces
        .get(entity)
        .expect("entities in Node EntityGrid should implement NodePiece");

    let glyph = if position == 0 {
        node_piece_render_registry
            .get(node_piece.display_name())
            .cloned()
            .unwrap_or_else(|| UNKNOWN_NODE_PIECE.to_owned())
    } else {
        FILL_GLYPH.to_owned()
    };
    match team_opt {
        None => configuration.color_scheme().mon().apply(glyph),
        Some(Team::Enemy) => configuration.color_scheme().enemy_team().apply(glyph),
        Some(Team::Player) => configuration.color_scheme().player_team().apply(glyph),
    }
}
