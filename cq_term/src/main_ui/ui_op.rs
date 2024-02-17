use game_core::op::{Op, OpImplResult, OpRegistrar};

use crate::prelude::*;
use crate::TerminalWindow;

#[derive(Clone, Debug, Reflect)]
pub enum MainUiOp {
    SwitchScreen(Entity),
}

impl Op for MainUiOp {
    fn register_systems(mut registrar: OpRegistrar<Self>) {
        registrar.register_op(opsys_switch_screen);
    }

    fn system_index(&self) -> usize {
        0
    }
}

fn opsys_switch_screen(
    In((_id, op)): In<(Entity, MainUiOp)>,
    mut res_terminal_window: ResMut<TerminalWindow>,
) -> OpImplResult {
    let MainUiOp::SwitchScreen(id) = op;
    res_terminal_window.set_render_target(Some(id));
    Ok(Metadata::default())
}
