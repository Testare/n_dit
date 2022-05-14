use super::{Direction, Node, Point, GameState, Pickup, Team};

pub enum Event {
    G(EventRecord<GameEvent>),
    N(EventRecord<NodeEvent>),
}

type EventConstructor<T> = fn(EventRecord<T>)->Event;

pub trait EventSubtype: Sized {
    type Metadata;
    type State;

    const CONSTRUCTOR: EventConstructor<Self>;

    fn apply(&self, state: &mut Self::State) -> Result<Self::Metadata, EventErr>;
    // fn unapply(&self, metadata: Self::Metadata, state: &mut Self::State);
    fn is_durable(&self, metadata: Self::Metadata) -> bool;
    fn state_from_game_state(state: &mut GameState) -> Option<&mut Self::State>;
    // fn to_event(event_record: EventRecord<Self>) -> Event;

    fn apply_gs(self, game_state: &mut GameState) -> Result<Event, EventErr> {
        if let Some(state) = Self::state_from_game_state(game_state) {
            let metadata = self.apply(state)?;
            Ok(Self::CONSTRUCTOR(EventRecord {
                event: self,
                metadata
            }))
        } else {
            Err(EventErr::NoRelevantState)
        }
    }
}

pub struct EventRecord<E: EventSubtype> {
    event: E,
    metadata: E::Metadata,
}

pub enum GameEvent {
    NextPage,
    CloseNode,
    OpenNode,
}

impl EventSubtype for GameEvent {
    type Metadata = ();
    type State = GameState;

    const CONSTRUCTOR: EventConstructor<Self> = Event::G;

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

    fn state_from_game_state(state: &mut GameState) -> Option<&mut Self::State> {
        Some(state)
    }

}

pub enum NodeEvent {
    ActivateSprite(usize), // Starts using a unit.
    DeactivateSprite,      // Finishes using a unit
    FinishTurn,
    MoveActiveSprite(Direction),
    TakeSpriteAction(usize, Point),
}

pub enum EventErr {
    FailedEvent,
    NoRelevantState,
}

impl EventSubtype for NodeEvent {
    type Metadata = NodeEventMetadata;
    type State = Node;

    const CONSTRUCTOR: EventConstructor<Self> = Event::N;

    fn apply(&self, node: &mut Self::State) -> Result<Self::Metadata, EventErr> {
        match self {
            NodeEvent::ActivateSprite(sprite_index) => {
                if node.activate_sprite(*sprite_index) {
                    Ok(NodeEventMetadata {
                        dropped_squares: Vec::new(),
                        pickup: None, 
                        team: Team::PlayerTeam,
                    })
                } else {
                    Err(EventErr::FailedEvent)
                }
            }
            _=> Err(EventErr::FailedEvent)
        }
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

    fn state_from_game_state(state: &mut GameState) -> Option<&mut Self::State> {
        state.node_mut()
    }
}

pub struct NodeEventMetadata {
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