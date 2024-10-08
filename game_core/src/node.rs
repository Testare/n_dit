use crate::card::{Action, CardDefinition, Deck};
use crate::item::{Item, ItemOp};
use crate::op::{CoreOps, OpPlugin, OpResult};
use crate::prelude::*;
use crate::NDitCoreSet;

mod ai;
mod node_loading;
mod node_op;
mod rule;

pub use ai::{AiThread, NodeBattleIntelligence, SimpleAiCurioOrder};
use bevy::ecs::entity::{EntityHashMap, EntityMapper, MapEntities};
use bevy::ecs::reflect::ReflectMapEntities;
use getset::CopyGetters;
pub use node_loading::NodeScene;
pub use node_op::node_op_undo::NodeUndoStack;
pub use node_op::NodeOp;
pub use rule::AccessPointLoadingRule;
use serde::{Deserialize, Serialize};

use self::daddy::Daddy;

pub mod key {
    use typed_key::{typed_key, Key};

    use super::*;

    pub const ALL_TEAM_MEMBERS_READY: Key<bool> = typed_key!("all_team_members_ready");
    pub const CARD: Key<Entity> = typed_key!("card");
    pub const CLOSING_NODE: Key<bool> = typed_key!("closing_node");
    pub const CURIO: Key<Entity> = typed_key!("curio");
    pub const DEACTIVATED_CURIO: Key<Entity> = typed_key!("deactivated_curio");
    pub const DROPPED_SQUARE: Key<UVec2> = typed_key!("dropped_square");
    pub const EFFECTS: Key<Metadata> = typed_key!("effects");
    pub const FIRST_VICTORY: Key<bool> = typed_key!("first_victory");
    pub const MOVED_PIECES: Key<HashMap<Entity, u32>> = typed_key!("pieces_moved");
    pub const REPLACED_SQUARE: Key<bool> = typed_key!("replaced_square");
    pub const REPLACED_SQUARE_NEXT: Key<UVec2> = typed_key!("replaced_square_next");
    pub const NODE_ID: Key<Entity> = typed_key!("node_id");
    pub const PICKUP: Key<Pickup> = typed_key!("pickup");
    pub const PICKUPS: Key<Vec<Pickup>> = typed_key!("pickups");
    pub const PICKUP_ID: Key<Entity> = typed_key!("pickup_id");
    pub const REMAINING_MOVES: Key<u32> = typed_key!("remaining_moves");
    pub const RETURNED_CARDS: Key<Vec<Entity>> = typed_key!("returned_cards");
    pub const SELF_EFFECTS: Key<Metadata> = typed_key!("self_effects");
    pub const SKIPPED_ACTIVATION: Key<bool> = typed_key!("skipped_activate");
    pub const TAPPED: Key<bool> = typed_key!("tapped");
    pub const TARGET_POINT: Key<UVec2> = typed_key!("target_pt");
    pub const UNLOADED_CARD: Key<Entity> = typed_key!("unloaded_card");
    pub const VICTORY_STATUS: Key<VictoryStatus> = typed_key!("victory_status");
}

#[derive(Debug)]
pub struct NodePlugin {
    pub always_award_pickups: bool,
}

impl Default for NodePlugin {
    fn default() -> Self {
        Self {
            always_award_pickups: true,
        }
    }
}

impl Plugin for NodePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<NoOpAction>()
            .init_resource::<Daddy<Node>>()
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
            .register_type::<VictoryAward>()
            .register_type::<VictoryStatus>()
            .register_type::<rule::AccessPointLoadingRule>()
            // Internal collection types need to be registered too
            .register_type::<Vec<Entity>>()
            // .register_type::<HashMap<Entity, VictoryStatus>>()
            .register_type::<EntityHashMap<VictoryStatus>>()
            .register_type::<Option<Entity>>()
            .add_plugins((
                ai::NodeAiPlugin,
                node_loading::NodeLoadingPlugin,
                node_op::node_op_undo::NodeOpUndoPlugin::default(),
                OpPlugin::<NodeOp>::default(),
            ));

        if self.always_award_pickups {
            app.add_systems(
                Update,
                sys_grant_pickups_on_node_exit.in_set(NDitCoreSet::PostProcessCommands),
            );
        }
    }
}

/// Indicates a point a player can play cards
#[derive(Component, CopyGetters, Debug, Default, Reflect)]
#[reflect(Component, MapEntities)]
pub struct AccessPoint {
    #[getset(get_copy = "pub")]
    card: Option<Entity>, // Display card data to load
}

impl MapEntities for AccessPoint {
    fn map_entities<M: EntityMapper>(&mut self, entity_mapper: &mut M) {
        if let Some(id) = self.card {
            self.card = Some(entity_mapper.map_entity(id))
        }
    }
}

/// Indicates the current curio performing moving and/or performing an action
#[derive(Component, Debug, Default, Deref, DerefMut, Reflect)]
#[reflect(Component)]
pub struct ActiveCurio(pub Option<Entity>);

/// Indicates a pickup has been claimed by a player
#[derive(Component, CopyGetters, Debug, Reflect)]
#[reflect(Component, MapEntities)]
#[get_copy = "pub"]
pub struct Claimed {
    node_id: Entity,
    player: Entity,
}

impl MapEntities for Claimed {
    fn map_entities<M: EntityMapper>(&mut self, entity_mapper: &mut M) {
        self.node_id = entity_mapper.map_entity(self.node_id);
        self.player = entity_mapper.map_entity(self.player);
    }
}

/// Indicates a node piece capable of moving and performing actions
#[derive(Component, Debug, Default, Reflect)]
#[reflect(Component, MapEntities)]
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
    fn map_entities<M: EntityMapper>(&mut self, entity_mapper: &mut M) {
        if let Some(id) = self.card {
            self.card = Some(entity_mapper.map_entity(id))
        }
    }
}

/// Indicates the team whose turn it is
#[derive(Component, Debug, Deref, DerefMut, Deserialize, Reflect, Serialize)]
#[reflect(Component, Serialize, Deserialize, MapEntities)]
pub struct CurrentTurn(pub Entity);

impl FromWorld for CurrentTurn {
    fn from_world(world: &mut World) -> Self {
        CurrentTurn(world.spawn_empty().id())
    }
}

impl MapEntities for CurrentTurn {
    fn map_entities<M: EntityMapper>(&mut self, entity_mapper: &mut M) {
        self.0 = entity_mapper.map_entity(self.0)
    }
}

/// Indicates that this player is attempting to enter a specified node.
#[derive(Component, Debug, Default, Deref, DerefMut, Deserialize, Reflect, Serialize)]
#[reflect(Component, Serialize, Deserialize)]
pub struct EnteringNode(pub NodeId);

/// Indicates this Player is in the specified node
#[derive(Clone, Component, Debug, Default, Deref, DerefMut, Deserialize, Reflect, Serialize)]
#[reflect(Component, Serialize, Deserialize)]
pub struct ForNode(pub NodeId);

/// Indicates this Player is in the specified node
#[derive(Clone, Component, Copy, Debug, Deref, DerefMut, Reflect)]
#[reflect(Component, MapEntities)]
pub struct InNode(pub Entity);

impl FromWorld for InNode {
    fn from_world(world: &mut World) -> Self {
        Self(world.spawn_empty().id())
    }
}

impl MapEntities for InNode {
    fn map_entities<M: EntityMapper>(&mut self, entity_mapper: &mut M) {
        self.0 = entity_mapper.map_entity(self.0)
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

/// Indicates an entity to create a curio from, given a card
#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub enum CurioFromCard {
    Path(String),
    Handle(Handle<CardDefinition>),
}

/// A pickup of money. Is NOT a component
#[derive(Clone, Copy, Debug, Default, Deserialize, Reflect, Serialize)]
pub struct Mon(pub u32);

#[derive(Component, Debug, Default, Deref, DerefMut, Reflect)]
#[reflect(Component)]
pub struct MovesTaken(pub u32);

/// Indicates a Node entity
/// Note that multiple Node instances at runtime can exist for the same [NodeId],
/// so that multiple players can play different instances of the same node.
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

impl std::fmt::Display for NodeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.set.as_str(), self.num)
    }
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

impl From<SetId> for NodeId {
    fn from(value: SetId) -> Self {
        NodeId::new(value.set(), value.num())
    }
}

#[derive(Component, Debug, Deserialize, Default, Reflect, Serialize)]
pub enum NodeLoadStatus {
    #[default]
    Loading,
    Loaded,
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
#[reflect(Component, MapEntities)]
pub struct OnTeam(pub Entity);

impl FromWorld for OnTeam {
    fn from_world(world: &mut World) -> Self {
        Self(world.spawn_empty().id())
    }
}

impl MapEntities for OnTeam {
    fn map_entities<M: EntityMapper>(&mut self, entity_mapper: &mut M) {
        self.0 = entity_mapper.map_entity(self.0)
    }
}

/// A pickup, typically found as a Node Piece. Enumerates different types
/// of pickups
#[derive(Clone, Component, Debug, Default, Deserialize, Reflect, Serialize)]
#[reflect(Component)]
pub enum Pickup {
    Mon(Mon),     // Money
    Card(String), // A new card to play
    Item(Entity), // A non-card item
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
///
/// Contains a list of cards and where they have been played to.
#[derive(Component, Debug, Default, Reflect)]
#[reflect(Component)]
pub struct PlayedCards(HashMap<Entity, Vec<Entity>>);

impl PlayedCards {
    fn num_played(&self, card_id: Entity) -> u32 {
        self.0
            .get(&card_id)
            .map(|played_list| played_list.len() as u32)
            .unwrap_or_default()
    }

    pub fn can_be_played(&self, deck: &Deck, card_id: Entity) -> bool {
        self.num_played(card_id) < deck.count_of_card(card_id)
    }

    pub fn can_be_withdrawn(&self, card_id: Entity, location: Entity) -> bool {
        self.0
            .get(&card_id)
            .map(|location_list| location_list.contains(&location))
            .unwrap_or(false)
    }

    pub fn remaining_count(&self, deck: &Deck, card_id: Entity) -> u32 {
        deck.count_of_card(card_id)
            .saturating_sub(self.num_played(card_id))
    }

    pub fn play_card_to(&mut self, deck: &Deck, card_id: Entity, location: Entity) -> bool {
        if self.can_be_played(deck, card_id) {
            let location_list = self.0.entry(card_id).or_default();
            location_list.push(location);
            true
        } else {
            false
        }
    }

    pub fn withdraw_card_from(&mut self, card_id: Entity, location: Entity) -> bool {
        self.0
            .get_mut(&card_id)
            .and_then(|location_list| {
                let idx = location_list.iter().position(|loc| *loc == location)?;
                location_list.remove(idx);
                Some(true)
            })
            .unwrap_or(false)
    }

    pub fn clear_location(&mut self, location: Entity) -> Vec<Entity> {
        self.0
            .iter_mut()
            .flat_map(|(card_id, location_list)| {
                let original_size = location_list.len();
                location_list.retain(|loc| *loc != location);
                std::iter::repeat(*card_id).take(original_size - location_list.len())
            })
            .collect()
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
/// TODO Probably should be changd to a component on Team entities
#[derive(Clone, Component, Debug, Deserialize, Default, Deref, DerefMut, Reflect, Serialize)]
#[reflect_value(Component, Deserialize, MapEntities, Serialize)] // Has to be reflect_value until this issue is solved: https://github.com/bevyengine/bevy/issues/10995
pub struct TeamStatus(EntityHashMap<VictoryStatus>);

impl MapEntities for TeamStatus {
    fn map_entities<M: EntityMapper>(&mut self, entity_mapper: &mut M) {
        self.0 = self
            .0
            .drain()
            .map(|(id, status)| (entity_mapper.map_entity(id), status))
            .collect();
    }
}

impl TeamStatus {
    fn is_decided(&self, team_id: Entity) -> bool {
        if let Some(status) = self.0.get(&team_id) {
            status.is_decided()
        } else {
            true // Why not?
        }
    }
}

/// Node component, listing the teams that belong to it
#[derive(Component, Debug, Default, Deref, DerefMut, Reflect)]
#[reflect(Component, MapEntities)]
pub struct Teams(pub Vec<Entity>);

impl MapEntities for Teams {
    fn map_entities<M: EntityMapper>(&mut self, entity_mapper: &mut M) {
        self.0 = self
            .0
            .iter()
            .map(|id| entity_mapper.map_entity(*id))
            .collect();
    }
}

#[derive(Component, Debug, Reflect)]
#[reflect(Component, MapEntities)]
pub struct VictoryAward(pub Entity);

impl MapEntities for VictoryAward {
    fn map_entities<M: EntityMapper>(&mut self, entity_mapper: &mut M) {
        self.0 = entity_mapper.map_entity(self.0);
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Reflect, Serialize)]
pub enum VictoryStatus {
    Undecided,
    Loss,
    Victory,
    PerfectVictory,
}

impl VictoryStatus {
    pub fn is_decided(&self) -> bool {
        !matches!(self, VictoryStatus::Undecided)
    }

    pub fn is_undecided(&self) -> bool {
        matches!(self, VictoryStatus::Undecided)
    }

    pub fn is_victorious(&self) -> bool {
        matches!(self, VictoryStatus::Victory | VictoryStatus::PerfectVictory)
    }
}

pub fn sys_grant_pickups_on_node_exit(
    asset_server: Res<AssetServer>,
    mut evr_node_op: EventReader<OpResult<NodeOp>>,
    mut res_core_ops: ResMut<CoreOps>,
) {
    for node_op_result in evr_node_op.read() {
        if let OpResult {
            source,
            op: NodeOp::QuitNode(_),
            result: Ok(metadata),
        } = node_op_result
        {
            log::debug!("{source:?} quit with {metadata:?}");
            match metadata.get_optional(key::PICKUPS) {
                Err(e) => {
                    log::error!("Error retrieving pickups: {e}");
                },
                Ok(None) => {},
                Ok(Some(pickups)) => {
                    for pickup in pickups.into_iter() {
                        let item = match pickup {
                            Pickup::Card(card_path) => Item::Card(asset_server.load(card_path)),
                            Pickup::Mon(mon_val) => Item::Mon(mon_val.0),
                            Pickup::MacGuffin => {
                                continue;
                            },
                            Pickup::Item(_item_id) => {
                                unimplemented!()
                            },
                        };
                        res_core_ops.request(*source, ItemOp::AddItem { item, refund: 0 });
                    }
                },
            }
        }
    }
}
