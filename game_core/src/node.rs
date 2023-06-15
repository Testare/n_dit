use crate::card::{ActionEffect, Deck};
use crate::prelude::*;

mod node_op;
mod rule;

use getset::CopyGetters;
pub use node_op::{access_point_actions, NodeOp};


#[derive(Component, Debug, FromReflect, Reflect)]
pub enum NodePhase {
    Loading,
    Setup {
        ready_players: HashSet<Entity>,
    },
    Play,
}

#[derive(Component, Debug, FromReflect, Reflect)]
pub struct PlayerTurn(Entity);

#[derive(Component, FromReflect, Reflect)]
pub struct Node;

/// Indicates this Player is in the node
#[derive(Component, Debug, Deref, DerefMut)]
pub struct InNode(pub Entity);

#[derive(Component, Reflect, getset::Getters)]
pub struct NodePiece {
    #[getset(get = "pub")]
    display_id: String,
}

#[derive(FromReflect, Reflect)]
pub struct Mon(pub u32);

#[derive(Component, Reflect)]
pub enum Pickup {
    Mon(Mon),
    Card(Entity),
    Item(Entity),
}

#[derive(Component, CopyGetters, Reflect, Default)]
pub struct AccessPoint {
    #[getset(get_copy = "pub")]
    card: Option<Entity>, // Display card data to load
}

#[derive(Component)]
pub struct PlayedCards {
    played_card_decks: Vec<Option<Deck>>,
}

#[derive(Component, Reflect)]
pub struct Curio {
    // owner: Entity, // Potential replacement for Team mechanism
    card: Option<Entity>,
    name: String,
}

#[derive(Component, Debug, Default, Deref, DerefMut, FromReflect, Reflect)]
pub struct MovesTaken(pub u32);

#[derive(Component, Debug, Default, Deref, DerefMut, FromReflect, Reflect)]
pub struct IsTapped(pub bool);

// Should it be "IsActivated" or should a node have an "ActivatedPiece"
#[derive(Component, Debug, Default, Deref, DerefMut, FromReflect, Reflect)]
pub struct ActivatedPiece(Option<Entity>);

#[derive(Clone, Component, Debug, FromReflect, Reflect)]
pub enum Team {
    Enemy,
    Player,
}

impl NodePiece {
    pub fn new(display_name: &str) -> Self {
        NodePiece {
            display_id: display_name.to_owned(),
        }
    }
}

impl Curio {
    pub fn new<S: Into<String>>(name: S) -> Self {
        Curio {
            name: name.into(),
            card: None,
        }
    }

    pub fn new_with_card<S: Into<String>>(name: S, card: Entity) -> Self {
        Curio {
            name: name.into(),
            card: Some(card),
        }
    }

    pub fn name(&self) -> &str {
        self.name.as_str()
    }
}
