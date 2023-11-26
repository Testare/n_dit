use crate::card::{Action, Deck};
use crate::opv2::OpPlugin;
use crate::prelude::*;

mod ai;
mod node_op;
mod rule;

pub use ai::{AiThread, NodeBattleIntelligence, SimpleAiCurioOrder};
use bevy::ecs::entity::MapEntities;
use getset::CopyGetters;
pub use node_op::NodeOp;
pub use rule::AccessPointLoadingRule;
use serde::{Deserialize, Serialize};

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
    pub const EFFECTS: Key<Metadata> = typed_key!("effects");
    pub const SELF_EFFECTS: Key<Metadata> = typed_key!("self_effects");
    pub const ALL_TEAM_MEMBERS_READY: Key<bool> = typed_key!("all_team_members_ready");
    pub const DEACTIVATED_CURIO: Key<Entity> = typed_key!("deactivated_curio");
}

#[derive(Debug)]
pub struct NodePlugin;

impl Plugin for NodePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<NoOpAction>()
            .register_type::<AccessPoint>()
            .register_type::<ActiveCurio>()
            .register_type::<Curio>()
            .register_type::<CurrentTurn>()
            .register_type::<InNode>()
            .register_type::<IsReadyToGo>()
            .register_type::<IsTapped>()
            .register_type::<Mon>()
            .register_type::<MovesTaken>()
            .register_type::<Node>()
            .register_type::<NodeId>()
            .register_type::<NodePiece>()
            .register_type::<OnTeam>()
            .register_type::<Pickup>()
            .register_type::<PlayedCards>()
            .register_type::<PreventNoOp>()
            .register_type::<Team>()
            .register_type::<TeamColor>()
            .register_type::<TeamPhase>()
            .register_type::<TeamStatus>()
            .register_type::<Teams>()
            .register_type::<VictoryStatus>()
            .add_plugins((ai::NodeAiPlugin, OpPlugin::<NodeOp>::default()));
    }
}

/// Indicates a point a player can play cards
#[derive(Component, CopyGetters, Debug, Default, Reflect)]
#[reflect(Component)]
pub struct AccessPoint {
    #[getset(get_copy = "pub")]
    card: Option<Entity>, // Display card data to load
}

impl MapEntities for AccessPoint {
    fn map_entities(&mut self, entity_mapper: &mut bevy::ecs::entity::EntityMapper) {
        if let Some(id) = self.card {
            self.card = Some(entity_mapper.get_or_reserve(id))
        }
    }
}

/// Indicates the current curio performing moving and/or performing an action
#[derive(Component, Debug, Default, Deref, DerefMut, Reflect)]
#[reflect(Component)]
pub struct ActiveCurio(pub Option<Entity>);

/// Indicates a node piece capable of moving and performing actions
#[derive(Component, Debug, Default, Reflect)]
#[reflect(Component)]
pub struct Curio {
    // owner: Entity, // Potential replacement for Team mechanism
    card: Option<Entity>,
    name: String,
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

impl MapEntities for Curio {
    fn map_entities(&mut self, entity_mapper: &mut bevy::ecs::entity::EntityMapper) {
        if let Some(id) = self.card {
            self.card = Some(entity_mapper.get_or_reserve(id))
        }
    }
}

/// Indicates the team whose turn it is
#[derive(Component, Debug, Deref, DerefMut, Deserialize, Reflect, Serialize)]
#[reflect(Component, Serialize, Deserialize)]
pub struct CurrentTurn(pub Entity);

impl FromWorld for CurrentTurn {
    fn from_world(world: &mut World) -> Self {
        CurrentTurn(world.spawn_empty().id())
    }
}

/// Indicates this Player is in the specified node
#[derive(Component, Debug, Default, Deref, DerefMut, Deserialize, Reflect, Serialize)]
#[reflect(Component, Serialize, Deserialize)]
pub struct ForNode(pub NodeId);

/// Indicates this Player is in the specified node
#[derive(Component, Debug, Deref, DerefMut, Reflect)]
#[reflect(Component)]
pub struct InNode(pub Entity);

impl FromWorld for InNode {
    fn from_world(world: &mut World) -> Self {
        Self(world.spawn_empty().id())
    }
}

impl MapEntities for InNode {
    fn map_entities(&mut self, entity_mapper: &mut bevy::ecs::entity::EntityMapper) {
        self.0 = entity_mapper.get_or_reserve(self.0)
    }
}

/// Indicates a player has finished setting up
#[derive(Component, Debug, Default, Deref, DerefMut, Reflect)]
#[reflect(Component)]
pub struct IsReadyToGo(pub bool);

/// Indicates a Curio has already moved or performed its action
#[derive(Component, Debug, Default, Deref, DerefMut, Reflect)]
#[reflect(Component)]
pub struct IsTapped(pub bool);

/// A pickup of money. Is NOT a component
#[derive(Debug, Default, Deserialize, Reflect, Serialize)]
pub struct Mon(pub u32);

#[derive(Component, Debug, Default, Deref, DerefMut, Reflect)]
#[reflect(Component)]
pub struct MovesTaken(pub u32);

/// Indicates a Node entity
#[derive(Component, Debug, Default, Deserialize, Reflect, Serialize)]
#[reflect(Component, Serialize, Deserialize)]
pub struct Node(pub NodeId);

/// Unique identifier for a Node.
#[derive(Clone, Component, Debug, Default, Deserialize, Hash, PartialEq, Reflect, Serialize)]
#[reflect(Deserialize, Serialize)]
pub struct NodeId {
    /// Nodes are within sets of up to 32 nodes
    set: String,
    /// Number of node in series,
    num: u32,
}

impl NodeId {
    /// ## Panics
    /// Panics if num is 32 or more
    pub fn new<S: ToString>(set: S, num: u32) -> Self {
        debug_assert!(num < 32, "Node ID has invalid value: [{}] >= 32", num);
        Self {
            num,
            set: set.to_string(),
        }
    }

    pub fn num(&self) -> u32 {
        self.num
    }

    pub fn set(&self) -> &str {
        self.set.as_str()
    }

    pub fn num_flag(&self) -> u32 {
        1 << self.num
    }
}

/// Indicates a piece that is loaded into a Node
#[derive(
    Component, Debug, Deserialize, Default, Reflect, getset::Getters, getset::Setters, Serialize,
)]
#[reflect(Component, Serialize, Deserialize)]
pub struct NodePiece {
    #[getset(get = "pub", set = "pub")]
    display_id: String,
}

impl NodePiece {
    pub fn new(display_id: &str) -> Self {
        NodePiece {
            display_id: display_id.to_owned(),
        }
    }
}

/// Resource to identify the default "No Action" action.
#[derive(Clone, Debug, Deref, Resource)]
pub struct NoOpAction(Handle<Action>);

impl FromWorld for NoOpAction {
    fn from_world(world: &mut World) -> Self {
        let mut assets = world
            .get_resource_mut::<Assets<Action>>()
            .expect("unable to load no op action");
        NoOpAction(assets.add(Action::default()))
    }
}

/// Indicates which team a node piece or player belongs to.
#[derive(Clone, Component, Copy, Debug, Deref, DerefMut, Eq, PartialEq, Reflect)]
#[reflect(Component)]
pub struct OnTeam(pub Entity);

impl FromWorld for OnTeam {
    fn from_world(world: &mut World) -> Self {
        Self(world.spawn_empty().id())
    }
}

impl MapEntities for OnTeam {
    fn map_entities(&mut self, entity_mapper: &mut bevy::ecs::entity::EntityMapper) {
        self.0 = entity_mapper.get_or_reserve(self.0)
    }
}

/// A pickup, typically found as a Node Piece. Enumerates different types
/// of pickups
#[derive(Component, Debug, Default, Deserialize, Reflect, Serialize)]
#[reflect(Component)]
pub enum Pickup {
    Mon(Mon),     // Money
    Card(Entity), // A new card to play
    Item(Entity), // An item
    #[default]
    MacGuffin, // A token of some sort, usually a victory condition
}

impl Pickup {
    pub fn default_diplay_id(&self) -> &'static str {
        match self {
            Self::Mon(_) => "pickup:mon",
            Self::Card(_) => "pickup:card",
            Self::Item(_) => "pickup:item",
            Self::MacGuffin => "pickup:macguffin",
        }
    }
}

/// Player Component, indicates which cards they've played already.
#[derive(Component, Debug, Default, Deref, DerefMut, Reflect)]
#[reflect(Component)]
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

// Might be moved to some combination of Rules and Tags
/// Indicates that this entity has to take an action once activated.
#[derive(Copy, Clone, Component, Debug, Default, Reflect)]
#[reflect(Component)]
pub struct PreventNoOp;

/// Marker component for a Team entity
#[derive(Component, Debug, Default, Reflect)]
#[reflect(Component)]
pub struct Team;

/// Originally used to color curios by team color. Might
/// change it later so that a team's pieces are outlined
/// in a color. Might also change it to just use a color
/// definition from Charmi.
#[derive(Component, Debug, Default, Reflect)]
#[reflect(Component)]
pub enum TeamColor {
    #[default]
    Red,
    Blue,
}

/// Indicates what phase of the game the team is in.
/// * Setup - Putting their pieces on the board
/// * Play - Usual play
#[derive(Clone, Component, Copy, Debug, Default, PartialEq, Reflect)]
#[reflect(Component)]
pub enum TeamPhase {
    #[default]
    Setup,
    Play,
}

/// Lists the victory status of each team in the node.
/// TODO Should be changd to a component on Team entities
#[derive(Component, Debug, Default, Deref, DerefMut, Reflect)]
#[reflect(Component)]
pub struct TeamStatus(pub HashMap<Entity, VictoryStatus>);

impl MapEntities for TeamStatus {
    fn map_entities(&mut self, entity_mapper: &mut bevy::ecs::entity::EntityMapper) {
        self.0 = self
            .0
            .iter()
            .map(|(id, status)| (entity_mapper.get_or_reserve(*id), *status))
            .collect();
    }
}

/// Node component, listing the teams that belong to it
#[derive(Component, Debug, Default, Deref, DerefMut, Reflect)]
#[reflect(Component)]
pub struct Teams(pub Vec<Entity>);

impl MapEntities for Teams {
    fn map_entities(&mut self, entity_mapper: &mut bevy::ecs::entity::EntityMapper) {
        self.0 = self
            .0
            .iter()
            .map(|id| entity_mapper.get_or_reserve(*id))
            .collect();
    }
}

#[derive(Clone, Copy, Debug, Reflect)]
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
