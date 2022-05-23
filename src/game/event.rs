use super::{GameChange, GameState, NodeChange, ChangeErr, StateChange};

// For now this will just be an alias.
// Perhaps in the future, we will replace with a catch-all "GameError"
pub type EventErr = ChangeErr;

#[derive(Debug, Clone)]
pub enum Event {
    G(usize, GameChange, <GameChange as StateChange>::Metadata),
    N(usize, NodeChange, <NodeChange as StateChange>::Metadata),
}

/// Used to help with converting a StateChange trait object into an Event, which can be serialized/deserialized/managed/etc
#[derive(Debug, Clone, Copy)]
pub enum Change {
    G(GameChange),
    N(NodeChange),
}

impl Change {
    fn apply_change<C:StateChange>(e: &C, game_state: &mut GameState) -> Result<C::Metadata, EventErr> {
        if let Some(state) = C::state_from_game_state(game_state) {
            e.apply(state)
        } else {
            Err(EventErr::NoRelevantState)
        }
    }

    pub(super) fn apply(self, id: usize, game_state: &mut GameState) -> Result<Event, EventErr> {
        match self {
            Self::G(ge) => {
                let ge_meta: <GameChange as StateChange>::Metadata = Self::apply_change(&ge, game_state)?;
                Ok(Event::G(id, ge, ge_meta))
            },
            Self::N(ne) => {
                let ne_meta: <NodeChange as StateChange>::Metadata = Self::apply_change(&ne, game_state)?;
                Ok(Event::N(id, ne, ne_meta))
            }
        }
    }
}

impl From<NodeChange> for Change {
    fn from(node_change: NodeChange) -> Self {
        Change::N(node_change)
    }
}

impl From<GameChange> for Change {
    fn from(game_change: GameChange) -> Self {
        Change::G(game_change)
    }
}