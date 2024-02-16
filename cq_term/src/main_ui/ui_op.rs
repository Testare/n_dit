use game_core::op::{Op, OpImplResult, OpRegistrar};

use crate::prelude::*;
use crate::TerminalWindow;

#[derive(Clone, Debug, Reflect)]
pub enum UiOp {
    SwitchScreen(Entity),
}

impl Op for UiOp {
    fn register_systems(mut registrar: OpRegistrar<Self>) {
        registrar.register_op(opsys_switch_screen);
    }

    fn system_index(&self) -> usize {
        0
    }
}

fn opsys_switch_screen(
    In((_id, op)): In<(Entity, UiOp)>,
    mut res_terminal_window: ResMut<TerminalWindow>,
) -> OpImplResult {
    let UiOp::SwitchScreen(id) = op;
    res_terminal_window.set_render_target(Some(id));
    Ok(Metadata::default())
}
