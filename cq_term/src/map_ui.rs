use crate::prelude::*;
use crate::render::{TerminalRendering, RENDER_TTY_SCHEDULE};
use charmi::CharacterMapImage;
use game_core::map::Map;

pub struct MapUiPlugin;

impl Plugin for MapUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            RENDER_TTY_SCHEDULE,
            render_map

        );
        
    }
}

#[derive(Clone, Component, Default, Deref, DerefMut, Reflect)]
#[reflect(Component)]
pub struct MapBackground(pub Handle<CharacterMapImage>);

fn render_map(
    ast_charmi: Res<Assets<CharacterMapImage>>,
    mut maps: Query<(AsDeref<MapBackground>, &mut TerminalRendering)>,
) {
    for (background_handle, mut tr) in maps.iter_mut() {
        let charmi = ast_charmi.get(background_handle);
        if let Some(charmi) = charmi {
            tr.update_charmie(charmi.clone());
        }
    }
}