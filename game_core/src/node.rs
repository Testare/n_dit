use crate::card::Deck;
use crate::prelude::*;

mod node_op;
mod rule;

use getset::CopyGetters;
pub use node_op::{access_point_ops, ready_to_go_ops, NodeOp};
pub use rule::AccessPointLoadingRule;

#[derive(Component, Debug, FromReflect, Reflect)]
pub struct PlayerTurn(Entity);

#[derive(Component, FromReflect, Reflect)]
pub struct Node;

/// Indicates this Player is in the node
#[derive(Component, Debug, Deref, DerefMut)]
pub struct InNode(pub Entity);

#[derive(Component, Reflect, getset::Getters, getset::Setters)]
pub struct NodePiece {
    #[getset(get = "pub", set = "pub")]
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

#[derive(Component, Default, Deref, DerefMut)]
pub struct PlayedCards(HashMap<Entity, u32>);

impl PlayedCards {
    pub fn can_be_played(&self, deck: &Deck, card_id: Entity) -> bool {
        self.get(&card_id).copied().unwrap_or_default() < deck.count_of_card(card_id)
    }

    pub fn remaining_count(&self, deck: &Deck, card_id: Entity) -> u32 {
        deck.count_of_card(card_id)
            .saturating_sub(self.get(&card_id).copied().unwrap_or_default())
    }
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

#[derive(Component, Deref, DerefMut)]
pub struct OnTeam(pub Entity);

#[derive(Component, Deref, DerefMut)]
pub struct Teams(pub Vec<Entity>);

#[derive(Component)]
pub enum TeamPhase {
    Setup,
    Play,
}

#[derive(Component)]
pub struct ReadyToGo(pub bool);

#[derive(Component)]
pub struct NodeTeam;

#[derive(Component)]
pub enum TeamColor {
    Red,
    Blue,
}

impl NodePiece {
    pub fn new(display_id: &str) -> Self {
        NodePiece {
            display_id: display_id.to_owned(),
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
