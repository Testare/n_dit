use super::{GameEvent, GameState, NodeEvent};

pub enum Event {
    G(EventRecord<GameEvent>),
    N(EventRecord<NodeEvent>),
}

pub struct EventRecord<E: EventSubtype> {
    event: E,
    metadata: E::Metadata,
}

pub enum EventErr {
    FailedEvent,
    NoRelevantState,
}

pub type EventConstructor<T> = fn(EventRecord<T>) -> Event;

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
                metadata,
            }))
        } else {
            Err(EventErr::NoRelevantState)
        }
    }
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
