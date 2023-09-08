use crate::card::{Action, ActionTarget, Deck, Description};
use crate::prelude::*;
use crate::NDitCoreSet;

mod ai;
mod node_op;
mod rule;

pub use ai::{AiThread, NodeBattleIntelligence, SimpleAiCurioOrder};
use getset::CopyGetters;
pub use node_op::{access_point_ops, curio_ops, ready_to_go_ops, NodeOp};
pub use rule::AccessPointLoadingRule;
use serde::{Deserialize, Serialize};

use self::node_op::end_turn_op;

pub mod key {
    use typed_key::{typed_key, Key};

    use super::*;

    pub const NODE_ID: Key<Entity> = typed_key!("node_id");
    pub const CURIO: Key<Entity> = typed_key!("curio");
    pub const TAPPED: Key<bool> = typed_key!("tapped");
    pub const PICKUP: Key<Pickup> = typed_key!("pickup");
    pub const DROPPED_SQUARE: Key<UVec2> = typed_key!("dropped_square");
    pub const REMAINING_MOVES: Key<u32> = typed_key!("remaining_moves");
    pub const MOVED_PIECES: Key<HashMap<Entity, u32>> = typed_key!("pieces_moved");
    pub const TARGET_POINT: Key<UVec2> = typed_key!("target_pt");
    pub const CARD: Key<Entity> = typed_key!("card");
    pub const ALL_TEAM_MEMBERS_READY: Key<bool> = typed_key!("all_team_members_ready");
}
pub struct NodePlugin;

#[derive(Component, Debug, Reflect)]
pub struct PlayerTurn(Entity);

#[derive(Component, Reflect)]
pub struct Node;

/// Indicates this Player is in the node
#[derive(Component, Debug, Deref, DerefMut)]
pub struct InNode(pub Entity);

#[derive(Component, Reflect, getset::Getters, getset::Setters)]
pub struct NodePiece {
    #[getset(get = "pub", set = "pub")]
    display_id: String,
}

#[derive(Debug, Deserialize, Reflect, Serialize)]
pub struct Mon(pub u32);

#[derive(Component, Debug, Deserialize, Reflect, Serialize)]
pub enum Pickup {
    Mon(Mon),
    Card(Entity),
    Item(Entity),
}

impl Pickup {
    pub fn default_diplay_id(&self) -> &'static str {
        match self {
            Self::Mon(_) => "pickup:mon",
            Self::Card(_) => "pickup:card",
            Self::Item(_) => "pickup:item",
        }
    }
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

#[derive(Component, Debug, Default, Deref, DerefMut, Reflect)]
pub struct MovesTaken(pub u32);

#[derive(Component, Debug, Default, Deref, DerefMut, Reflect)]
pub struct IsTapped(pub bool);

#[derive(Clone, Component, Copy, Deref, DerefMut, Eq, PartialEq)]
pub struct OnTeam(pub Entity);

#[derive(Component, Deref, DerefMut)]
pub struct CurrentTurn(pub Entity);

#[derive(Component, Default, Deref, DerefMut)]
pub struct ActiveCurio(pub Option<Entity>);

#[derive(Component, Debug, Deref, DerefMut)]
pub struct Teams(pub Vec<Entity>);

#[derive(Component, Debug, Deref, DerefMut, Reflect)]
pub struct TeamStatus(pub HashMap<Entity, VictoryStatus>);

#[derive(Debug, Reflect)]
pub enum VictoryStatus {
    Undecided,
    Loss,
    Victory,
    PerfectVictory,
}

impl VictoryStatus {
    pub fn is_undecided(&self) -> bool {
        matches!(self, VictoryStatus::Undecided)
    }
}

#[derive(Component, Debug, PartialEq)]
pub enum TeamPhase {
    Setup,
    Play,
}

#[derive(Component)]
pub struct IsReadyToGo(pub bool);

#[derive(Component)]
pub struct Team;

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

#[derive(Copy, Clone, Debug, Deref, Resource)]
pub struct NoOpAction(pub Entity);

// Might be moved to some combination of Rules and Tags
#[derive(Copy, Clone, Component)]
pub struct PreventNoOp;

impl FromWorld for NoOpAction {
    fn from_world(world: &mut World) -> Self {
        NoOpAction(
            world
                .spawn((
                    Action {
                        name: "No Action".to_owned(),
                    },
                    ActionTarget::None,
                    Description::new("End movement and do nothing"),
                ))
                .id(),
        )
    }
}

impl Plugin for NodePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<NoOpAction>()
            .add_systems(
                Update,
                (access_point_ops, ready_to_go_ops, curio_ops, end_turn_op)
                    .in_set(NDitCoreSet::ProcessCommands),
            )
            .add_plugins(ai::NodeAiPlugin);
    }
}
