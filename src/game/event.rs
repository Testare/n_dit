use super::{Direction, Point, GameState, Pickup};

enum Event {
    G(EventRecord<GameEvent>),
    N(EventRecord<NodeEvent>),
}


trait EventSubtype {
    type Metadata;
    fn apply(&self, state: &mut GameState) -> Result<Option<Self::Metadata>, EventErr>;
}

struct EventRecord<E: EventSubtype> {
    event: E,
    metadata: Option<E::Metadata>,
}

enum GameEvent {
    NextPage, // Used for animations

    CloseNode, // TODO NOCOMMIT figure out whether CloseNode is a GameEvent or NodeEvent
}




impl EventSubtype for GameEvent {
    type Metadata = ();

    fn apply(&self, state: &mut GameState) -> Result<Option<Self::Metadata>, EventErr> {
        Ok(Some(()))
    }
}


enum NodeEvent {
    ActivateSprite(usize), // Starts using a unit.
    DeactivateSprite,      // Finishes using a unit
    MoveActiveSprite(Direction),
    TakeSpriteAction(usize, Point),
}

enum EventErr {
    FailedEvent
}

impl EventSubtype for NodeEvent {
    type Metadata = NodeEventMetadata;

    fn apply(&self, state: &mut GameState) -> Result<Option<Self::Metadata>, EventErr> {
        Err(EventErr::FailedEvent)
    }
}

struct NodeEventMetadata {
    /// Movement or action caused these squares to drop. 
    /// We should do testing to make sure they are recorded in the order of ebing dropped off and res
    dropped_squares: Vec<Point>, 
    // An item was picked up during movement
    pickup: Option<Pickup>, 
}

trait GameEventListener {
    fn apply<E: EventSubtype>(&mut self, event: E, metadata: E::Metadata) {
        match event.into() {
            Event::N(e) => self.node_event(e),
            Event::G(e) => self.game_event(e),
        }
    }

    fn node_event(&mut self, e: EventRecord<NodeEvent>) {} 
    fn game_event(&mut self, g: EventRecord<GameEvent>) {}

}