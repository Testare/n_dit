use super::error::{Error, Result};
use super::{GameChange, GameState, NodeChange, StateChange};

// For now this will just be an alias.
// Perhaps in the future, we will replace with a catch-all "GameError"

#[derive(Debug, Clone)]
pub enum Event {
    G {
        id: usize,
        change: GameChange,
        metadata: <GameChange as StateChange>::Metadata,
    },
    N {
        id: usize,
        change: NodeChange,
        metadata: <NodeChange as StateChange>::Metadata,
    },
}

impl Event {
    pub fn id(&self) -> usize {
        match self {
            Event::G { id, .. } => *id,
            Event::N { id, .. } => *id,
        }
    }
}

/// Used to help with converting a StateChange trait object into an Event, which can be serialized/deserialized/managed/etc
#[derive(Debug, Clone, Copy)]
pub enum Change {
    G(GameChange),
    N(NodeChange),
}

impl Change {
    fn apply_change<C: StateChange>(e: &C, game_state: &mut GameState) -> Result<C::Metadata> {
        if let Some(state) = C::state_from_game_state(game_state) {
            e.apply(state)
        } else {
            Err(Error::InvalidForContext(format!(
                "Requires context [{}]",
                C::STATE_NAME
            )))
        }
    }

    pub(super) fn apply(self, id: usize, game_state: &mut GameState) -> Result<Event> {
        match self {
            Self::G(change) => {
                let metadata: <GameChange as StateChange>::Metadata =
                    Self::apply_change(&change, game_state)?;
                Ok(Event::G {
                    id,
                    change,
                    metadata,
                })
            }
            Self::N(change) => {
                let metadata: <NodeChange as StateChange>::Metadata =
                    Self::apply_change(&change, game_state)?;
                Ok(Event::N {
                    id,
                    change,
                    metadata,
                })
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
