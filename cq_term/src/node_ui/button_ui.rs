use std::borrow::Cow;
use std::marker::PhantomData;

use bevy::ecs::query::Has;
use charmi::CharacterMapImage;
use crossterm::style::ContentStyle;
use game_core::node::NodeOp;
use game_core::op::OpSubtype;
use game_core::player::ForPlayer;
use game_core::NDitCoreSet;
use unicode_width::UnicodeWidthStr;

use super::NodeUiOp;
use crate::layout::{CalculatedSizeTty, LayoutMouseTargetDisabled};
use crate::prelude::*;
use crate::render::TerminalRendering;

#[derive(Component, Reflect)]
struct ButtonUi {
    text: String,
    short_text: String,
    // Color?
}

#[derive(Component)]
pub struct FlexibleTextUi {
    style: ContentStyle,
    text: String,
}

#[derive(Component, Reflect)]
pub enum TextUiBorder {
    Brackets,
    Parenthesis,
}

#[derive(Component)]
pub struct DisabledTextEffect(ContentStyle);

fn sys_render_flexible_text(
    mut buttons: Query<(
        &FlexibleTextUi,
        &CalculatedSizeTty,
        &mut TerminalRendering,
        Has<LayoutMouseTargetDisabled>,
        Option<&TextUiBorder>,
        Option<&DisabledTextEffect>,
    )>,
) {
    for (text_ui, size, mut rendering, disabled, text_ui_border, disabled_text_effect) in
        buttons.iter_mut()
    {
        let borders_len = if text_ui_border.is_some() { 2 } else { 0 };
        // TODO This is not unicode safe
        let text_len = size.width().saturating_sub(borders_len).max(1);
        let render_text = if text_len < text_ui.text.width() {
            text_ui.text.chars().take(text_len).collect()
        } else {
            Cow::from(text_ui.text.as_str())
        };
        if render_text.width() > text_len {
            panic!("UNICODE buttons not supported yet.");
        }
        let render_text = match text_ui_border {
            None => render_text,
            Some(TextUiBorder::Brackets) => Cow::from(format!("[{}]", render_text)),
            Some(TextUiBorder::Parenthesis) => Cow::from(format!("[{}]", render_text)),
        };
        let next_rendering =
            CharacterMapImage::new().with_row(|row| row.with_text(render_text, &text_ui.style));
        rendering.update_charmie(next_rendering)
    }
    // TODO allow configuring short buttons to be always on
}
