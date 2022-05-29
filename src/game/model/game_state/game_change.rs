use super::super::super::abstractions::StateChange;
use super::super::super::error::{Error, Result};
use super::super::animation::Animation;
use crate::GameState;

#[derive(Debug, Clone, Copy)]
pub enum GameChange {
    NextPage,
    CloseNode,
    OpenNode,
}

impl StateChange for GameChange {
    type Metadata = ();
    type State = GameState;

    const STATE_NAME: &'static str = "GAME";

    fn apply(&self, state: &mut GameState) -> Result<Self::Metadata> {
        use GameChange::*;
        match self {
            NextPage => Animation::next(state).map_err(Error::NotPossibleForState),
            _ => unimplemented!("Game changes have not been implemented yet"),
        }
    }

    fn unapply(&self, _: &(), _state: &mut GameState) -> Result<()> {
        unimplemented!("No undo game actions implemented, though it might be good for animations")
    }

    fn is_durable(&self, _: &()) -> bool {
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
