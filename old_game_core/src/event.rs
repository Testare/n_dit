use serde::{Deserialize, Serialize};

use super::error::{Error, ErrorMsg as _, Result};
use super::{GameChange, GameState, NodeChange, StateChange};

// TODO In the future, turn this into a trait and use typetag crate
#[derive(Debug, Clone, Deserialize, Serialize)]
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
    fn undo_change<C: StateChange>(
        change: &C,
        metadata: &C::Metadata,
        game_state: &mut GameState,
    ) -> Result<()> {
        if let Some(state) = C::state_from_game_state(game_state) {
            change.unapply(metadata, state)
        } else {
            format!("Undo requires context [{}]", C::STATE_NAME).fail_critical()
        }
    }

    pub fn id(&self) -> usize {
        match self {
            Event::G { id, .. } => *id,
            Event::N { id, .. } => *id,
        }
    }

    pub fn into_change(self) -> Change {
        match self {
            Event::G { change, .. } => Change::G(change),
            Event::N { change, .. } => Change::N(change),
        }
    }

    pub fn is_durable(&self) -> bool {
        match self {
            Event::G {
                change, metadata, ..
            } => change.is_durable(metadata),
            Event::N {
                change, metadata, ..
            } => change.is_durable(metadata),
        }
    }

    pub(super) fn undo(&self, game_state: &mut GameState) -> Result<()> {
        match self {
            Event::G {
                change, metadata, ..
            } => Self::undo_change(change, metadata, game_state),
            Event::N {
                change, metadata, ..
            } => Self::undo_change(change, metadata, game_state),
        }
    }
}

/// Used to help with converting a StateChange trait object into an Event, which can be serialized/deserialized/managed/etc
#[derive(Debug, Clone)]
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
            },
            Self::N(change) => {
                let metadata: <NodeChange as StateChange>::Metadata =
                    Self::apply_change(&change, game_state)?;
                Ok(Event::N {
                    id,
                    change,
                    metadata,
                })
            },
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
