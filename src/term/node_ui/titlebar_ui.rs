use game_core::prelude::*;

use super::NodeUi;
use crate::term::render::{RenderTtySet, UpdateRendering};

#[derive(Component, Default)]
pub struct TitleBarUi;

pub fn render_title_bar_system(
    mut commands: Commands,
    render_title_bar: Query<Entity, With<TitleBarUi>>,
) {
    let rendered_text = vec!["n_dit".to_owned()];
    for id in render_title_bar.iter() {
        commands
            .get_entity(id)
            .update_rendering(rendered_text.clone());
    }
}

impl Plugin for TitleBarUi {
    fn build(&self, app: &mut App) {
        app.add_system(render_title_bar_system.in_set(RenderTtySet::PostCalculateLayout));
    }
}

impl NodeUi for TitleBarUi {
    type UiBundle = ();
    type UiPlugin = Self;

    fn ui_bundle() -> Self::UiBundle {
        ()
    }
}
