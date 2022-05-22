use super::GameState;

// Not quite sure where this should live yet, so for now it gets its own file

pub trait StateChange: Sized {
    type Metadata;
    type State;

    fn apply(&self, state: &mut Self::State) -> Result<Self::Metadata, ChangeErr>;
    // fn unapply(&self, metadata: Self::Metadata, state: &mut Self::State);
    fn is_durable(&self, metadata: Self::Metadata) -> bool;
    fn state_from_game_state(state: &mut GameState) -> Option<&mut Self::State>;
}

#[derive(Debug)]
pub enum ChangeErr {
    FailedEvent,
    NoRelevantState,
}