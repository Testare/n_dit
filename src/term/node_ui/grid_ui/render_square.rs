use game_core::prelude::*;
use game_core::{IsTapped, NodePiece, Team};

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
    node_pieces: &Query<super::NodePieceQ>,
    node_piece_render_registry: &GlyphRegistry,
    configuration: &DrawConfiguration,
) -> (UiFormat, String) {
    let node_piece = node_pieces
        .get(entity)
        .expect("entities in Node EntityGrid should implement NodePiece");

    let glyph = if position == 0 {
        node_piece_render_registry
            .get(node_piece.piece.display_id())
            .cloned()
            .unwrap_or_else(|| UNKNOWN_NODE_PIECE.to_owned())
    } else {
        FILL_GLYPH.to_owned()
    };
    let format = if node_piece.access_point.is_some() {
        configuration.color_scheme().access_point()
    } else {
        match (node_piece.is_tapped, node_piece.team) {
            (_, None) => configuration.color_scheme().mon(),
            (_, Some(Team::Enemy)) => configuration.color_scheme().enemy_team(),
            (Some(IsTapped(true)), Some(Team::Player)) => {
                configuration.color_scheme().player_team_tapped()
            },
            (_, Some(Team::Player)) => configuration.color_scheme().player_team(),
        }
    };
    (format, glyph)
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
