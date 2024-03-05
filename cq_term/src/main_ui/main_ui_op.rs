use game_core::op::{Op, OpImplResult, OpRegistrar};

use super::MainUi;
use crate::prelude::*;

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
    mut commands: Commands,
    mut q_main_ui: Query<(Entity, AsDerefMut<MainUi>)>,
) -> OpImplResult {
    let MainUiOp::SwitchScreen(next_screen) = op;
    if let Ok((main_ui_id, mut last_screen)) = q_main_ui.get_single_mut() {
        let mut ui_id_commands = commands.entity(main_ui_id);
        if Some(next_screen) != *last_screen.deref() {
            if let Some(last_screen) = *last_screen {
                ui_id_commands.remove_children(&[last_screen]);
            }
            *last_screen = Some(next_screen);
            ui_id_commands.add_child(next_screen);
        }
    }
    Ok(Metadata::default())
}
