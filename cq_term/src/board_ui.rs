use charmi::CharacterMapImage;

use crate::prelude::*;
use crate::render::{TerminalRendering, RENDER_TTY_SCHEDULE};

pub struct BoardUiPlugin;

impl Plugin for BoardUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(RENDER_TTY_SCHEDULE, render_map);
    }
}

#[derive(Clone, Component, Default, Deref, DerefMut, Reflect)]
#[reflect(Component)]
pub struct BoardBackground(pub Handle<CharacterMapImage>);

fn render_map(
    ast_charmi: Res<Assets<CharacterMapImage>>,
    mut maps: Query<(AsDeref<BoardBackground>, &mut TerminalRendering)>,
) {
    for (background_handle, mut tr) in maps.iter_mut() {
        let charmi = ast_charmi.get(background_handle);
        if let Some(charmi) = charmi {
            tr.update_charmie(charmi.clone());
        }
    }
}
