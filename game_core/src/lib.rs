pub mod bam;
pub mod board;
pub mod card;
pub mod common;
pub mod configuration;
pub mod dialog;
pub mod entity_grid;
pub mod item;
pub mod node;
pub mod op;
pub mod player;
pub mod prelude;
pub mod quest;
pub mod registry;
pub mod saving;
pub mod shop;

use op::CoreOps;
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
    ProcessUiOps,
    PostProcessUiOps,
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
pub struct NDitCorePlugin;

impl Plugin for NDitCorePlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(
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
                NDitCoreSet::ProcessUiOps,
                NDitCoreSet::PostProcessUiOps,
            )
                .chain(),
        )
        .add_plugins((
            bam::BamPlugin,
            player::PlayerPlugin,
            saving::SavePlugin,
            board::BoardPlugin,
            card::CardPlugin,
            dialog::DialogPlugin,
            item::ItemPlugin,
            entity_grid::EntityGridSupportPlugin,
            node::NodePlugin::default(),
            op::OpExecutorPlugin::<CoreOps>::new(Update, Some(NDitCoreSet::ProcessCommands)),
            quest::QuestPlugin,
            shop::ShopPlugin,
            registry::RegistryPlugin,
        ));
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
