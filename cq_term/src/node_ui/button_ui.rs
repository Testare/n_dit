use std::marker::PhantomData;

use charmi::CharacterMapImage;
use game_core::node::NodeOp;
use game_core::op::OpSubtype;
use game_core::player::ForPlayer;
use game_core::NDitCoreSet;
use unicode_width::UnicodeWidthStr;

use super::NodeUiOp;
use crate::layout::CalculatedSizeTty;
use crate::prelude::*;
use crate::render::TerminalRendering;

#[derive(Component, Reflect)]
struct ButtonUi<O: OpSubtype> {
    op: O,
    text: String,
    short_text: String,
    // Color?
}

struct ButtonUiPlugin<O> {
    _phantom_data: PhantomData<O>,
}

impl<O: OpSubtype + Send + Sync + 'static> Plugin for ButtonUiPlugin<O> {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (sys_mouse_button_ui::<O>,).in_set(NDitCoreSet::ProcessInputs),
        );
    }
}

fn sys_mouse_button_ui<O: OpSubtype>(
    buttons: Query<(&ButtonUi<O>, &ForPlayer)>,
    mut ev_op: EventWriter<Op<O>>,
) {
    for (button, for_player) in buttons.iter() {
        button.op.clone().for_p(**for_player).send(&mut ev_op)
    }
}

fn sys_render_button<O: OpSubtype>(
    mut buttons: Query<(&ButtonUi<O>, &CalculatedSizeTty, &mut TerminalRendering)>,
) {
    for (button, size, mut rendering) in buttons.iter_mut() {
        let text = if size.x < button.text.width() as u32 + 2 {
            &button.text
        } else {
            &button.short_text
        };
        let next_rendering =
            CharacterMapImage::new().with_row(|mut row| row.with_plain_text(format!("[{}]", text)));
        rendering.update_charmie(next_rendering)
    }
    // TODO allow configuring short buttons to be always on
}
