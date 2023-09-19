use crossterm::style::ContentStyle;
use game_core::node::ActiveCurio;
use game_core::prelude::*;
use game_core::registry::Reg;

use crate::configuration::DrawConfiguration;
use crate::node_ui::NodeGlyph;

const FILL_GLYPH: &'static str = "[]";

pub fn render_square(
    position: usize,
    entity: Entity,
    active_curio: &ActiveCurio,
    node_pieces: &Query<super::NodePieceQ>,
    reg_glyph: &Reg<NodeGlyph>,
    configuration: &DrawConfiguration,
) -> (ContentStyle, String) {
    let node_piece = node_pieces
        .get(entity)
        .expect("entities in Node EntityGrid should implement NodePiece");
    let node_glyph = reg_glyph
        .get(node_piece.piece.display_id().as_str())
        .cloned()
        .unwrap_or_default();
    let head_glyph = node_glyph.glyph();
    let glyph_style = node_glyph.style();

    let chosen_format = if node_piece.access_point.is_some() {
        configuration.color_scheme().access_point().to_content_style()
    } else if node_piece
        .is_tapped
        .map(|is_tapped| **is_tapped)
        .unwrap_or_default()
    {
        configuration.color_scheme().player_team_tapped().to_content_style()
    } else if active_curio
        .map(|curio_id| curio_id == entity && position == 0)
        .unwrap_or_default()
    {
        configuration.color_scheme().player_team_active().to_content_style()
    } else {
        glyph_style
    };
    let glyph = if position == 0 {
        head_glyph
    } else {
        FILL_GLYPH.to_owned()
    };
    (chosen_format, glyph)
}