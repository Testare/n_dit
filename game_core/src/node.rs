use super::NDitError;
use crate::prelude::*;
use old_game_core::GridMap;

#[derive(Component, FromReflect, Reflect)]
pub struct Node;

#[derive(Component, Reflect, getset::Getters)]
pub struct NodePiece {
    #[getset(get="pub")]
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

#[derive(Component, Debug, Deref, FromReflect, Reflect)]
pub struct Actions(Vec<Action>);

#[derive(Debug, FromReflect, Reflect, )]
pub struct Action {
    pub name: String,
    pub range: usize,
    // effect
    // desc
}
    

#[derive(Clone, Component, Debug, FromReflect, Reflect)]
pub enum Team {
    Enemy,
    Player,
}

impl NodePiece {
    pub fn new(display_name: &str) -> Self {
        NodePiece {
            display_name: display_name.to_owned(),
        }
    }
}

impl Actions {
    pub fn new(actions: Vec<Action>) -> Self {
        Actions(actions)
    }
}