use super::NDitError;
use bevy::prelude::*;
use old_game_core::GridMap;

#[derive(Component, FromReflect, Reflect)]
pub struct Node;

#[derive(Component, Deref, DerefMut, FromReflect, Reflect)]
pub struct EntityGrid {
    // Re-implement grid_map.rs to store entity references directly
    grid: GridMap<Entity>,
}

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

impl EntityGrid {
    pub fn new_from_shape(shape: &str) -> Result<EntityGrid, NDitError> {
        let grid_map = GridMap::from_shape_string(shape);
        if let Err(e) = grid_map {
            return Err(NDitError::DecodeError {
                encoded_string: shape.to_string(),
                decode_error: format!("{:?}", e),
            });
        }
        Ok(EntityGrid {
            grid: grid_map.unwrap(),
        })
    }
}
