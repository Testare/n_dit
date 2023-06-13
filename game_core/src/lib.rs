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

pub struct Op<O> {
    pub op: O,
    pub player: usize,
}

impl<O> Op<O> {
    pub fn new<const P: usize>(op: O) -> Self {
        Op { op, player: P }
    }

    pub fn op(&self) -> &O {
        &self.op
    }

    pub fn player(&self) -> usize {
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
            .add_systems((node::access_point_actions,).in_set(NDitCoreSet::ProcessCommands));
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Direction {
    North,
    East,
    South,
    West,
}

impl Direction {
    pub const ALL_DIRECTIONS: [Direction; 4] = [
        Direction::North,
        Direction::East,
        Direction::South,
        Direction::West,
    ];
}

impl std::ops::Add<Direction> for UVec2 {
    type Output = UVec2;
    fn add(self, rhs: Direction) -> Self::Output {
        let UVec2 { x, y } = self;
        match rhs {
            Direction::North => UVec2 {
                x,
                y: y.saturating_sub(1),
            },
            Direction::East => UVec2 { x: x + 1, y },
            Direction::South => UVec2 { x, y: y + 1 },
            Direction::West => UVec2 {
                x: x.saturating_sub(1),
                y,
            },
        }
    }
}
