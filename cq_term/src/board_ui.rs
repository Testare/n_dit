use charmi::CharacterMapImage;

use crate::prelude::*;
use crate::render::{TerminalRendering, RENDER_TTY_SCHEDULE};

pub struct BoardUiPlugin;

impl Plugin for BoardUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(RENDER_TTY_SCHEDULE, (sys_render_board,));
    }
}

#[derive(Clone, Component, Deref, DerefMut, Reflect)]
#[reflect(Component)]
pub struct BoardUi(pub Entity); // track Board

impl FromWorld for BoardUi {
    fn from_world(world: &mut World) -> Self {
        BoardUi(world.spawn_empty().id())
    }
}

#[derive(Clone, Component, Default, Deref, DerefMut, Reflect)]
#[reflect(Component)]
pub struct BoardBackground(pub Handle<CharacterMapImage>);

fn sys_render_board(
    ast_charmi: Res<Assets<CharacterMapImage>>,
    mut map_uis: Query<(
        AsDeref<BoardBackground>,
        &mut TerminalRendering,
        AsDeref<BoardUi>,
    )>,
) {
    for (background_handle, mut tr, _board_id) in map_uis.iter_mut() {
        let charmi = ast_charmi.get(background_handle);
        if let Some(charmi) = charmi {
            tr.update_charmie(charmi.clone());
        }
    }
}
