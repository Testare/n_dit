use game_core::prelude::*;
use game_core::{NodePiece, Team};
use super::registry::GlyphRegistry;
use crate::term::configuration::DrawConfiguration;

const UNKNOWN_NODE_PIECE: &'static str = "??";
const FILL_GLYPH: &'static str = "[]";

pub fn render_square(
    position: usize,
    entity: Entity,
    node_pieces: &Query<(&NodePiece, Option<&Team>)>,
    node_piece_render_registry: &GlyphRegistry,
    configuration: &DrawConfiguration,
) -> String {
    /*let string = match sprite {
        Sprite::AccessPoint(_) => String::from("&&"),
        Sprite::Pickup(pickup) => configuration
            .color_scheme()
            .mon()
            .apply(pickup.square_display()),
        Sprite::Curio(curio) => {
            if position == 0 {
                String::from(curio.display())
            } else if false {
                // Logic to format the last square differently if position + 1 == max_size and the
                // curio is selected, so that you can tell that moving will not grow the curio.
                String::from("[]")
            } else {
                match configuration.tail_appearance() {
                    FillMethod::NoFill => String::from("  "),
                    FillMethod::Brackets => String::from("[]"),
                    FillMethod::DotFill => String::from(".."),
                    FillMethod::HeadCopy => String::from(curio.display()),
                    FillMethod::Sequence => {
                        format!("{:02}", position)
                    }
                }
            }
        }
    };
    style(sprite, node, key, position, configuration).apply(string)*/

    let (node_piece, team_opt) = node_pieces.get(entity).expect("entities in Node EntityGrid should implement NodePiece");
    
    let glyph = if position == 0 {
        node_piece_render_registry.get(node_piece.display_name()).cloned().unwrap_or_else(||UNKNOWN_NODE_PIECE.to_owned())
    } else {
        FILL_GLYPH.to_owned()
    };
    match team_opt {
        None => configuration.color_scheme().mon().apply(glyph),
        Some(Team::Enemy) => configuration.color_scheme().enemy_team().apply(glyph),
        Some(Team::Player) => configuration.color_scheme().player_team().apply(glyph),
    }
}
