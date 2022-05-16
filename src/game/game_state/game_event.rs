use super::super::event::*;
use crate::GameState;

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
