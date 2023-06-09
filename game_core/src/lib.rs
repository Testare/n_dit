pub mod prelude;
mod entity_grid;
pub mod card;
pub mod node;
pub mod player;

pub use node::*;

use self::prelude::*;
use thiserror::Error;

pub use node::{Mon, Node, NodePiece};

pub use entity_grid::EntityGrid;

#[derive(Debug, Error)]
pub enum NDitError {
    #[error("attempt to decode string [{encoded_string}] but encountered error [{decode_error}]")]
    DecodeError {
        encoded_string: String,
        decode_error: String,
    },
}

pub struct NDitCorePlugin;

impl Plugin for NDitCorePlugin {
    fn build(&self, app: &mut App) {

    }
}


#[derive(Clone, Copy, Debug)]
pub enum Direction {
    North,
    East,
    South,
    West
}

impl Direction {
    pub const ALL_DIRECTIONS: [Direction; 4]= [
        Direction::North,
        Direction::East,
        Direction::South,
        Direction::West,
    ];
}

impl std::ops::Add<Direction> for UVec2 {
    type Output = UVec2;
    fn add(self, rhs: Direction) -> Self::Output {
        let UVec2 {x, y} = self;
        match rhs {
            Direction::North => UVec2 {
                x,
                y: y.saturating_sub(1),
            },
            Direction::East => UVec2 {
                x: x + 1,
                y, 
            },
            Direction::South => UVec2 {
                x, 
                y: y + 1, 
            },
            Direction::West => UVec2 { 
                x: x.saturating_sub(1), 
                y,
            },
        }
    }
}