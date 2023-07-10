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
    ProcessCommandsFlush,
    PostProcessCommands,
}

#[derive(Debug, Error)]
pub enum NDitError {
    #[error("attempt to decode string [{encoded_string}] but encountered error [{decode_error}]")]
    DecodeError {
        encoded_string: String,
        decode_error: String,
    },
}

#[derive(Clone, Debug, Event)]
pub struct Op<O> {
    pub op: O,
    pub player: Entity,
}

pub trait OpSubtype: Clone {
    type Metadata;
    type Error;
}

#[derive(Clone, Debug, Event, getset::Getters)]
pub struct OpResult<O: OpSubtype> {
    #[getset(get = "pub")]
    source: Op<O>,
    #[getset(get = "pub")]
    result: Result<O::Metadata, O::Error>,
}

impl<O: OpSubtype> OpResult<O> {
    fn new(source: &Op<O>, result: Result<O::Metadata, O::Error>) -> Self {
        OpResult {
            source: source.clone(),
            result,
        }
    }
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
            .add_event::<OpResult<node::NodeOp>>()
            .configure_sets(
                First,
                (NDitCoreSet::RawInputs.before(NDitCoreSet::ProcessInputs),),
            )
            .configure_sets(
                PreUpdate,
                (NDitCoreSet::ProcessInputs.before(NDitCoreSet::ProcessCommands),),
            )
            .configure_sets(
                Update,
                (
                    NDitCoreSet::ProcessCommands,
                    NDitCoreSet::ProcessCommandsFlush,
                    NDitCoreSet::PostProcessCommands,
                )
                    .chain(),
            )
            .add_systems(
                Update,
                (
                    apply_deferred.in_set(NDitCoreSet::ProcessCommandsFlush),
                    (card::sys_sort_decks,).in_set(NDitCoreSet::PostProcessCommands),
                ),
            )
            .add_plugin(node::NodePlugin);
    }
}

#[macro_export]
macro_rules! get_assert {
    ($id:expr, $q:expr) => {{
        let temp_result = ($q).get($id);
        debug_assert!(
            temp_result.is_ok(),
            "expected query get failed {:?}",
            temp_result.err()
        );
        if let Err(e) = temp_result {
            log::error!("[line: {}] expected query get failed [{:?}]", line!(), e);
        }
        temp_result.ok()
    }};
    ($id:expr, $q:expr, $closure:expr) => {{
        let temp_result = ($q).get($id);
        debug_assert!(
            temp_result.is_ok(),
            "expected query get failed {:?}",
            temp_result.err()
        );
        if let Err(e) = temp_result {
            log::error!("[line: {}] expected query get failed [{:?}]", line!(), e);
        }
        temp_result.ok().and_then($closure)
    }};
}

#[macro_export]
macro_rules! get_assert_mut {
    ($id:expr, $q:expr) => {{
        let temp_result = ($q).get_mut($id);
        debug_assert!(
            temp_result.is_ok(),
            "expected query get failed {:?}",
            temp_result.err()
        );
        if let Err(e) = temp_result {
            log::error!("[line: {}] expected query get failed [{:?}]", line!(), e);
        }
        temp_result.ok()
    }};
    ($id:expr, $q:expr, $block:expr) => {{
        let temp_result = ($q).get_mut($id);
        debug_assert!(
            temp_result.is_ok(),
            "expected query get failed {:?}",
            temp_result.err()
        );
        if let Err(e) = temp_result {
            log::error!("[line: {}] expected query get failed [{:?}]", line!(), e);
        }
        temp_result.ok().and_then($block)
    }};
}
