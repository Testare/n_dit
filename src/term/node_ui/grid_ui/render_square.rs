use game_core::node::ActiveCurio;
use game_core::prelude::*;

use super::super::registry::GlyphRegistry;
use crate::term::configuration::{DrawConfiguration, UiFormat};

const UNKNOWN_NODE_PIECE: &'static str = "??";
const FILL_GLYPH: &'static str = "[]";

/*fn style(
    sprite: &Sprite,
    node: &Node,
    key: usize,
    position: usize,
    draw_config: &DrawConfiguration,
) -> UiFormat {
    match sprite {
        Sprite::Pickup(_) => draw_config.color_scheme().mon(),
        Sprite::AccessPoint(_) => draw_config.color_scheme().access_point(),
        Sprite::Curio(curio) => match curio.team() {
            Team::PlayerTeam => {
                if node.active_curio_key() == Some(key) {
                    draw_config.color_scheme().player_team_active()
                } else if curio.tapped() && position == 0 {
                    draw_config.color_scheme().player_team_tapped()
                } else {
                    draw_config.color_scheme().player_team()
                }
            }
            Team::EnemyTeam => draw_config.color_scheme().enemy_team(),
        },
    }
}*/

pub fn render_square(
    position: usize,
    entity: Entity,
    active_curio: &ActiveCurio,
    node_pieces: &Query<super::NodePieceQ>,
    node_piece_render_registry: &GlyphRegistry,
    configuration: &DrawConfiguration,
) -> (UiFormat, String) {
    let node_piece = node_pieces
        .get(entity)
        .expect("entities in Node EntityGrid should implement NodePiece");
    let (head_glyph, glyph_format) = node_piece_render_registry
        .get(node_piece.piece.display_id())
        .cloned()
        .unwrap_or_else(|| (UNKNOWN_NODE_PIECE.to_owned(), UiFormat::NONE));
    let chosen_format = if node_piece.access_point.is_some() {
        configuration.color_scheme().access_point()
    } else if node_piece
        .is_tapped
        .map(|is_tapped| **is_tapped)
        .unwrap_or_default()
    {
        configuration.color_scheme().player_team_tapped()
    } else if active_curio
        .map(|curio_id| curio_id == entity && position == 0)
        .unwrap_or_default()
    {
        configuration.color_scheme().player_team_active()
    } else {
        glyph_format
    };
    let glyph = if position == 0 {
        head_glyph
    } else {
        FILL_GLYPH.to_owned()
    };
    (chosen_format, glyph)
}

// Might want to change this to just accept a mutable Write reference to make more effecient.
/*fn style(
    sprite: &Sprite,
    node: &Node,
    key: usize,
    position: usize,
    draw_config: &DrawConfiguration,
) -> UiFormat {
    match sprite {
        Sprite::Pickup(_) => draw_config.color_scheme().mon(),
        Sprite::AccessPoint(_) => draw_config.color_scheme().access_point(),
        Sprite::Curio(curio) => match curio.team() {
            Team::PlayerTeam => {
                if node.active_curio_key() == Some(key) {
                    draw_config.color_scheme().player_team_active()
                } else if curio.tapped() && position == 0 {
                    draw_config.color_scheme().player_team_tapped()
                } else {
                    draw_config.color_scheme().player_team()
                }
            }
            Team::EnemyTeam => draw_config.color_scheme().enemy_team(),
        },
    }
}*/
