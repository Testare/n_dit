use super::super::event::*;
use crate::{Direction, GameState, Node, Pickup, Point, Team};

pub enum NodeEvent {
    ActivateSprite(usize), // Starts using a unit.
    DeactivateSprite,      // Finishes using a unit
    FinishTurn,
    MoveActiveSprite(Direction),
    TakeSpriteAction(usize, Point),
}

type NodeEventResult = Result<NodeEventMetadata, EventErr>;

impl Node {
    fn activate_sprite_event(&mut self, sprite_index: usize) -> NodeEventResult {
        if self.activate_sprite(sprite_index) {
            Ok(NodeEventMetadata::for_team(self.active_team()))
        } else {
            Err(EventErr::FailedEvent)
        }
    }

    fn deactivate_sprite_event(&mut self) -> NodeEventResult {
        self.deactivate_sprite();
        Ok(NodeEventMetadata::for_team(self.active_team()))
    }
}

impl EventSubtype for NodeEvent {
    type Metadata = NodeEventMetadata;
    type State = Node;

    const CONSTRUCTOR: EventConstructor<Self> = Event::N;

    fn apply(&self, node: &mut Self::State) -> Result<Self::Metadata, EventErr> {
        match self {
            NodeEvent::ActivateSprite(sprite_index) => node.activate_sprite_event(*sprite_index),
            NodeEvent::DeactivateSprite => node.deactivate_sprite_event(),
            _ => Err(EventErr::FailedEvent),
        }
    }

    fn is_durable(&self, metadata: NodeEventMetadata) -> bool {
        if metadata.team.is_ai() {
            return false;
        }
        use NodeEvent::*;
        match self {
            DeactivateSprite | FinishTurn | TakeSpriteAction(_, _) => true,
            ActivateSprite(_) | MoveActiveSprite(_) => false,
        }
    }

    fn state_from_game_state(state: &mut GameState) -> Option<&mut Self::State> {
        state.node_mut()
    }
}

#[derive(Clone)]
pub struct NodeEventMetadata {
    /// Movement or action caused these squares to drop.
    /// We should do testing to make sure they are recorded in the order of ebing dropped off and res
    dropped_squares: Vec<Point>,
    // An item was picked up during movement
    pickup: Option<Pickup>,
    team: Team,
}

impl NodeEventMetadata {
    fn for_team(team: Team) -> NodeEventMetadata {
        NodeEventMetadata {
            team,
            pickup: None,
            dropped_squares: Vec::new(),
        }
    }

    fn with_pickup(mut self, pickup: Pickup) -> NodeEventMetadata {
        self.pickup = Some(pickup);
        self
    }

    fn with_dropped_squares(mut self, dropped_squares: Vec<Point>) -> NodeEventMetadata {
        self.dropped_squares = dropped_squares;
        self
    }
}
