use std::borrow::{Borrow, Cow};

use bevy::ecs::query::Has;
use charmi::CharacterMapImage;
use crossterm::style::{ContentStyle, Stylize};
use taffy::prelude::Size;
use taffy::style::Dimension;
use unicode_width::UnicodeWidthStr;

use crate::layout::{CalculatedSizeTty, LayoutMouseTarget, LayoutMouseTargetDisabled, StyleTty};
use crate::prelude::*;
use crate::render::{RenderTtySet, TerminalRendering, RENDER_TTY_SCHEDULE};

#[derive(Default)]
pub struct BaseUiPlugin;

impl Plugin for BaseUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            RENDER_TTY_SCHEDULE,
            (sys_render_flexible_text.in_set(RenderTtySet::RenderLayouts),),
        );
    }
}

#[derive(Bundle)]
pub struct ButtonUiBundle {
    pub name: Name,
    pub text_ui: FlexibleTextUi,
    pub borders: TextUiBorder,
    pub mouse_target: LayoutMouseTarget,
    pub disabled_effect: DisabledTextEffect,
    pub rendering: TerminalRendering,
    pub style_tty: StyleTty,
}

impl ButtonUiBundle {
    pub fn new<S: Borrow<str>>(text: S, style: ContentStyle) -> Self {
        ButtonUiBundle {
            name: Name::from(format!("Button ({})", text.borrow())),
            text_ui: FlexibleTextUi {
                style,
                text: text.borrow().to_string(),
            },
            borders: TextUiBorder::Brackets,
            mouse_target: LayoutMouseTarget,
            disabled_effect: DisabledTextEffect(ContentStyle::default().dark_grey()),
            rendering: TerminalRendering::default(),

            style_tty: StyleTty(taffy::prelude::Style {
                size: Size {
                    width: Dimension::Points(text.borrow().len() as f32 + 2.0),
                    height: Dimension::Points(1.0),
                },
                max_size: Size {
                    width: Dimension::Points(text.borrow().len() as f32 + 2.0),
                    height: Dimension::Points(1.0),
                },
                min_size: Size {
                    width: Dimension::Points(3.0),
                    height: Dimension::Points(1.0),
                },
                flex_grow: 0.0,
                flex_shrink: 1.0,
                ..default()
            }),
        }
    }
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

#[derive(Clone, Component, Deref, DerefMut)]
pub struct DisabledTextEffect(ContentStyle);

// TODO HoverTextEffect when mouse events supports it

pub fn sys_render_flexible_text(
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
            panic!("UNICODE text support coming soon!");
        }
        let render_text = match text_ui_border {
            None => render_text,
            Some(TextUiBorder::Brackets) => Cow::from(format!("[{}]", render_text)),
            Some(TextUiBorder::Parenthesis) => Cow::from(format!("[{}]", render_text)),
        };
        let mut next_rendering =
            CharacterMapImage::new().with_row(|row| row.with_text(render_text, &text_ui.style));
        if let (true, Some(effect)) = (disabled, disabled_text_effect) {
            next_rendering.apply_effect(&**effect);
        }
        rendering.update_charmie(next_rendering)
    }
    // TODO allow configuring short buttons to be always on
}
