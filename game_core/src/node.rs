use super::NDitError;
use crate::prelude::*;
use old_game_core::GridMap;

#[derive(Component, FromReflect, Reflect)]
pub struct Node;

#[derive(Component, Reflect)]
pub struct NodePiece {
    display_name: String,
}

#[derive(Component, Reflect)]
pub struct Mon(pub u32);

#[derive(Component, Reflect)]
struct AccessPoint {
    card: Entity, // Display card data to load
}

#[derive(Component, Reflect)]
struct Curio {
    max_size: usize,
    speed: usize,
    owner: Entity,
    card: Entity,
}

impl NodePiece {
    pub fn new(display_name: &str) -> Self {
        NodePiece {
            display_name: display_name.to_owned(),
        }
    }
}