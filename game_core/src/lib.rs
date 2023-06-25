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
                NDitCoreSet::ProcessInputs
                    .in_base_set(CoreSet::PreUpdate)
                    .after(NDitCoreSet::RawInputs),
                NDitCoreSet::ProcessCommands
                    .in_base_set(CoreSet::Update)
                    .after(NDitCoreSet::ProcessInputs),
                NDitCoreSet::ProcessCommandsFlush
                    .in_base_set(CoreSet::Update)
                    .after(NDitCoreSet::ProcessCommands),
                NDitCoreSet::PostProcessCommands
                    .in_base_set(CoreSet::Update)
                    .after(NDitCoreSet::ProcessCommandsFlush),
            ))
            .add_systems(
                (
                    node::access_point_ops,
                    node::ready_to_go_ops,
                    node::curio_ops,
                )
                    .in_set(NDitCoreSet::ProcessCommands),
            )
            .add_system(apply_system_buffers.in_set(NDitCoreSet::ProcessCommandsFlush))
            .add_systems((card::sys_sort_decks,).in_set(NDitCoreSet::PostProcessCommands));
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
    ($id:expr, $q:expr, $block:expr) => {{
        let temp_result = ($q).get($id);
        debug_assert!(
            temp_result.is_ok(),
            "expected query get failed {:?}",
            temp_result.err()
        );
        if let Err(e) = temp_result {
            log::error!("[line: {}] expected query get failed [{:?}]", line!(), e);
        }
        temp_result.ok().and_then(block)
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
