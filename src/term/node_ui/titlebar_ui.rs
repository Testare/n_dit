use game_core::prelude::*;

use crate::term::render::UpdateRendering;


#[derive(Component)]
pub struct TitleBarUi;

pub fn render_title_bar_system(
    mut commands: Commands,
    render_title_bar: Query<Entity, With<TitleBarUi>>,
) {
    let rendered_text = vec!["n_dit".to_owned()];
    if let Ok(id) = render_title_bar.get_single() {
        commands
            .get_entity(id)
            .update_rendering(rendered_text.clone());
    }
}
