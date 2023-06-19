pub mod card;
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

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Compass {
    North,
    East,
    South,
    West,
}

impl Compass {
    pub const ALL_DIRECTIONS: [Compass; 4] =
        [Compass::North, Compass::East, Compass::South, Compass::West];
}

impl std::ops::Add<Compass> for UVec2 {
    type Output = UVec2;
    fn add(self, rhs: Compass) -> Self::Output {
        let UVec2 { x, y } = self;
        match rhs {
            Compass::North => UVec2 {
                x,
                y: y.saturating_sub(1),
            },
            Compass::East => UVec2 { x: x + 1, y },
            Compass::South => UVec2 { x, y: y + 1 },
            Compass::West => UVec2 {
                x: x.saturating_sub(1),
                y,
            },
        }
    }
}
