use game_core::prelude::*;
use taffy::style::Dimension;

use super::NodeUi;
use crate::term::layout::{CalculatedSizeTty, StyleTty};
use crate::term::render::{RenderTtySet, UpdateRendering};

#[derive(Component, Debug, Default, Deref, DerefMut, FromReflect, Reflect)]
pub struct MessageBarUi(pub Vec<String>);

#[derive(Default)]
pub struct MessageBarUiPlugin;

pub fn style_message_bar(mut ui: Query<(&CalculatedSizeTty, &MessageBarUi, &mut StyleTty)>) {
    for (size, ui, mut style) in ui.iter_mut() {
        let height = Dimension::Points(if let Some(msg) = ui.first() {
            2.0 + textwrap::wrap(msg.as_str(), size.width()).len() as f32
        } else {
            1.0
        });
        if height != style.size.height {
            style.size.height = height;
        }
    }
}

pub fn render_message_bar(
    mut commands: Commands,
    ui: Query<(Entity, &MessageBarUi, &CalculatedSizeTty)>,
) {
    if let Ok((id, msgbar, size)) = ui.get_single() {
        let mut rendered_text: Vec<String> = vec![format!("{0:─<1$}", "─Messages", size.width())];
        if let Some(msg) = msgbar.first() {
            for line in textwrap::wrap(msg.as_str(), size.width()).into_iter() {
                rendered_text.push(line.to_string());
            }
            rendered_text.push("---Enter to continue---".to_owned());
        }
        commands
            .get_entity(id)
            .update_rendering(rendered_text.clone());
    }
}

impl Plugin for MessageBarUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems((
            style_message_bar.in_set(RenderTtySet::PreCalculateLayout),
            render_message_bar.in_set(RenderTtySet::PostCalculateLayout),
        ));
    }
}

impl NodeUi for MessageBarUi {
    type UiBundle = ();
    type UiPlugin = MessageBarUiPlugin;

    fn ui_bundle() -> Self::UiBundle {
        ()
    }
}
