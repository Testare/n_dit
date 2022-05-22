use super::super::{StateChange, event::EventErr};
use crate::GameState;
use super::super::animation::Animation;

#[derive(Debug, Clone, Copy)]
pub enum GameChange {
    NextPage,
    CloseNode,
    OpenNode,
}

impl StateChange for GameChange {
    type Metadata = ();
    type State = GameState;

    fn apply(&self, state: &mut GameState) -> Result<Self::Metadata, EventErr> {
        use GameChange::*;
        match self {
            NextPage => Animation::next(state).map_err(|_|EventErr::FailedEvent),
            _ => unimplemented!("Game changes have not been implemented yet")
        }
    }

    fn is_durable(&self, _: ()) -> bool {
        use GameChange::*;
        match self {
            NextPage => false,
            CloseNode | OpenNode => true,
        }
    }

    fn state_from_game_state(state: &mut GameState) -> Option<&mut Self::State> {
        Some(state)
    }
}
