use super::error::{Result};
use super::GameState;

// Not quite sure where this should live yet, so for now it gets its own file

pub trait StateChange: Sized {
    type Metadata;
    type State;

    const STATE_NAME: &'static str;

    fn apply(&self, state: &mut Self::State) -> Result<Self::Metadata>;
    fn unapply(&self, metadata: &Self::Metadata, state: &mut Self::State) -> Result<()>;
    fn is_durable(&self, metadata: &Self::Metadata) -> bool;
    fn state_from_game_state(state: &mut GameState) -> Option<&mut Self::State>;
}
