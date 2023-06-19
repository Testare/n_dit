pub mod card;
pub mod common;
mod entity_grid;
pub mod node;
pub mod player;
pub mod prelude;

// TODO no longer use these publicly, but have all itnerfaces one level deep?
use thiserror::Error;

use self::prelude::*;

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub enum NDitCoreSet {
    RawInputs,
    ProcessInputs,
    ProcessCommands,
}

#[derive(Debug, Error)]
pub enum NDitError {
    #[error("attempt to decode string [{encoded_string}] but encountered error [{decode_error}]")]
    DecodeError {
        encoded_string: String,
        decode_error: String,
    },
}

#[derive(Debug)]
pub struct Op<O> {
    pub op: O,
    pub player: Entity,
}

impl<O> Op<O> {
    pub fn new(player: Entity, op: O) -> Self {
        Op { op, player }
    }

    pub fn op(&self) -> &O {
        &self.op
    }

    pub fn player(&self) -> Entity {
        self.player
    }
}

pub struct NDitCorePlugin;

impl Plugin for NDitCorePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<Op<node::NodeOp>>()
            .configure_sets((
                NDitCoreSet::RawInputs.in_base_set(CoreSet::First),
                NDitCoreSet::ProcessInputs.in_base_set(CoreSet::PreUpdate),
                NDitCoreSet::ProcessCommands.in_base_set(CoreSet::Update),
            ))
            .add_systems(
                (
                    node::access_point_ops,
                    node::ready_to_go_ops,
                    node::curio_ops,
                )
                    .in_set(NDitCoreSet::ProcessCommands),
            );
    }
}
