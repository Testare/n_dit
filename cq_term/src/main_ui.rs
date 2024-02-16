mod ui_op;

use game_core::op::{OpExecutor, OpExecutorPlugin, OpPlugin};
use game_core::NDitCoreSet;
pub use ui_op::UiOp;

use crate::prelude::*;

#[derive(Debug, Default, Deref, DerefMut, Resource)]
pub struct UiOps(OpExecutor);

#[derive(Debug, Default)]
pub struct MainUiPlugin;

impl Plugin for MainUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            OpExecutorPlugin::<UiOps>::new(Update, Some(NDitCoreSet::ProcessUiOps)),
            OpPlugin::<UiOp>::default(),
        ));
    }
}
