use super::{Direction, Node, Point, GameState, Pickup, Team};

enum Event {
    G(EventRecord<GameEvent>),
    N(EventRecord<NodeEvent>),
}


trait EventSubtype {
    type Metadata;
    type State;
    fn apply(&self, state: &mut Self::State) -> Result<Self::Metadata, EventErr>;
    fn is_durable(&self, metadata: Self::Metadata) -> bool;
}

struct EventRecord<E: EventSubtype> {
    event: E,
    metadata: Option<E::Metadata>,
}

enum GameEvent {
    NextPage,
    CloseNode,
    OpenNode,
}

impl EventSubtype for GameEvent {
    type Metadata = ();
    type State = GameState;

    fn apply(&self, state: &mut GameState) -> Result<Self::Metadata, EventErr> {
        Ok(())
    }

    fn is_durable(&self, _: ()) -> bool {
        use GameEvent::*;
        match self {
            NextPage => false,
            CloseNode | OpenNode => true,
        }
    }

}

enum NodeEvent {
    ActivateSprite(usize), // Starts using a unit.
    DeactivateSprite,      // Finishes using a unit
    FinishTurn,
    MoveActiveSprite(Direction),
    TakeSpriteAction(usize, Point),
}

enum EventErr {
    FailedEvent
}

impl EventSubtype for NodeEvent {
    type Metadata = NodeEventMetadata;
    type State = Node;

    fn apply(&self, state: &mut Self::State) -> Result<Self::Metadata, EventErr> {
        Err(EventErr::FailedEvent)
    }

    fn is_durable(&self, metadata : NodeEventMetadata) -> bool {
        if metadata.team.is_ai() {
            return false;
        }
        use NodeEvent::*;
        match self {
            DeactivateSprite | FinishTurn | TakeSpriteAction(_, _) => true,
            ActivateSprite(_) | MoveActiveSprite(_) => false,
        }
    }
}

struct NodeEventMetadata {
    /// Movement or action caused these squares to drop. 
    /// We should do testing to make sure they are recorded in the order of ebing dropped off and res
    dropped_squares: Vec<Point>, 
    // An item was picked up during movement
    pickup: Option<Pickup>, 
    team: Team,
}

trait GameEventListener {
    fn apply(&mut self, event: Event) {
        match event {
            Event::N(e) => self.node_event(e),
            Event::G(e) => self.game_event(e),
        }
    }

    fn node_event(&mut self, e: EventRecord<NodeEvent>) {} 
    fn game_event(&mut self, g: EventRecord<GameEvent>) {}

}
