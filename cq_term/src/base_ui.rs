pub mod context_menu;
mod input_actions;
mod popup;

use std::borrow::{Borrow, Cow};

use bevy::ecs::query::Has;
use charmi::CharacterMapImage;
use crossterm::style::{ContentStyle, Stylize};
use game_core::NDitCoreSet;
use pad::PadStr;

use self::context_menu::ContextMenuPlugin;
pub use self::popup::*;
use crate::input_event::{
    MouseEventListener, MouseEventTty, MouseEventTtyDisabled, MouseEventTtyKind,
};
use crate::layout::{CalculatedSizeTty, StyleTty};
use crate::prelude::*;
use crate::render::{RenderTtySet, TerminalRendering, RENDER_TTY_SCHEDULE};

/// Represents a component that can be scrolled both horizontally and vertically
/// Currently, this scroll has to be used directly by the rendering systems individually
///
/// Later, we might change it so that those components will normally just render fully
/// and the scroll can be used to clip the image in layouts, to simplify render systems.
/// We can add a marker component used to indicate entities that will render the scrolled
/// layout themselves.
#[derive(Component, Debug, Default, Deref, DerefMut, Reflect)]
pub struct Scroll2d(pub UVec2);

/// Root layout
#[derive(Debug, Default)]
pub struct PaneRoot;

#[derive(Debug, Default)]
pub struct BaseUiPlugin;

impl Plugin for BaseUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, sys_apply_hover.in_set(NDitCoreSet::ProcessInputs))
            .add_systems(
                RENDER_TTY_SCHEDULE,
                (
                    sys_tooltip_on_hover.in_set(RenderTtySet::PreCalculateLayout),
                    sys_render_flexible_text.in_set(RenderTtySet::PostCalculateLayout),
                    sys_render_flexible_text_multiline.in_set(RenderTtySet::PostCalculateLayout),
                    popup::sys_render_popup_menu.in_set(RenderTtySet::PostCalculateLayout),
                    popup::sys_mouse_popup_menu.in_set(RenderTtySet::PostCalculateLayout),
                ),
            )
            .add_plugins(ContextMenuPlugin);
    }
}

#[derive(Component, Debug, Default, Deref, DerefMut)]
pub struct HoverPoint(Option<UVec2>);

#[derive(Component, Debug, Default)]
pub struct TooltipBar;

#[derive(Clone, Component, Debug, Default, Deref, DerefMut)]
pub struct Tooltip(Cow<'static, str>);

#[derive(Bundle, Debug)]
pub struct ButtonUiBundle {
    pub name: Name,
    pub text_ui: FlexibleTextUi,
    pub borders: TextUiBorder,
    pub mouse_target: MouseEventListener,
    pub disabled_effect: DisabledTextEffect,
    pub hover_point: HoverPoint,
    pub rendering: TerminalRendering,
    pub style_tty: StyleTty,
}

impl ButtonUiBundle {
    pub fn new<S: Borrow<str>>(text: S, text_style: ContentStyle) -> Self {
        use taffy::prelude::*;
        ButtonUiBundle {
            name: Name::from(format!("Button ({})", text.borrow())),
            text_ui: FlexibleTextUi {
                style: text_style,
                text: text.borrow().to_string(),
            },
            borders: TextUiBorder::Brackets,
            mouse_target: MouseEventListener,
            disabled_effect: DisabledTextEffect(ContentStyle::default().dark_grey()),
            rendering: TerminalRendering::default(),
            hover_point: HoverPoint::default(),
            style_tty: StyleTty(taffy::prelude::Style {
                size: Size {
                    width: length(text.borrow().len() as f32 + 2.0),
                    height: length(1.0),
                },
                max_size: Size {
                    width: length(text.borrow().len() as f32 + 2.0),
                    height: length(1.0),
                },
                min_size: Size {
                    width: length(3.0),
                    height: length(1.0),
                },
                flex_grow: 0.0,
                flex_shrink: 1.0,
                ..default()
            }),
        }
    }

    pub fn new_with_style_tty<S: Borrow<str>>(
        text: S,
        text_style: ContentStyle,
        style: StyleTty,
    ) -> Self {
        let mut init_bundle = Self::new(text, text_style);
        let taffy::prelude::Style {
            size: base_size,
            min_size: base_min_size,
            max_size: base_max_size,
            ..
        } = init_bundle.style_tty.0;
        let default_style = taffy::prelude::Style::default();
        let size = if style.0.size == default_style.size {
            base_size
        } else {
            style.0.size
        };
        let min_size = if style.0.size == default_style.size {
            base_min_size
        } else {
            style.0.min_size
        };
        let max_size = if style.0.size == default_style.size {
            base_max_size
        } else {
            style.0.max_size
        };
        init_bundle.style_tty = StyleTty(taffy::prelude::Style {
            size,
            min_size,
            max_size,
            ..style.0
        });
        init_bundle
    }
}

#[derive(Component, Debug)]
pub struct FlexibleTextUi {
    pub style: ContentStyle,
    pub text: String,
}

#[derive(Component, Debug)]
pub struct FlexibleTextUiMultiline {
    pub style: ContentStyle,
    pub text: String,
}

#[derive(Component, Debug, Reflect)]
pub enum TextUiBorder {
    Brackets,
    Parenthesis,
}

#[derive(Clone, Component, Debug, Deref, DerefMut)]
pub struct DisabledTextEffect(ContentStyle);

impl Tooltip {
    pub fn new<S: Into<Cow<'static, str>>>(tooltip: S) -> Self {
        Tooltip(tooltip.into())
    }
}

pub fn sys_render_flexible_text(
    mut buttons: Query<(
        &FlexibleTextUi,
        &CalculatedSizeTty,
        &mut TerminalRendering,
        Has<MouseEventTtyDisabled>,
        Option<AsDeref<HoverPoint>>,
        Option<&TextUiBorder>,
        Option<&DisabledTextEffect>,
    )>,
) {
    for (
        text_ui,
        size,
        mut rendering,
        disabled,
        hover_point,
        text_ui_border,
        disabled_text_effect,
    ) in buttons.iter_mut()
    {
        let borders_len = if text_ui_border.is_some() { 2 } else { 0 };
        let text_len = size.width().saturating_sub(borders_len).max(1);
        let render_text = text_ui.text.with_exact_width(text_len);
        let render_text = match text_ui_border {
            None => render_text,
            Some(TextUiBorder::Brackets) => format!("[{}]", render_text),
            Some(TextUiBorder::Parenthesis) => format!("[{}]", render_text),
        };
        let mut next_rendering =
            CharacterMapImage::new().with_row(|row| row.with_text(render_text, &text_ui.style));
        if let (true, Some(effect)) = (disabled, disabled_text_effect) {
            next_rendering.apply_effect(effect);
        } else if matches!(hover_point, Some(Some(_))) {
            next_rendering.apply_effect(&ContentStyle::new().reverse());
        }
        rendering.update_charmie(next_rendering)
    }
    // TODO allow configuring short buttons to be always on
}

// TOOD this can probably be merged with above and simply add a "Wrapping" marker component
pub fn sys_render_flexible_text_multiline(
    mut buttons: Query<
        (
            &FlexibleTextUiMultiline,
            &CalculatedSizeTty,
            &mut TerminalRendering,
            Has<MouseEventTtyDisabled>,
            Option<AsDeref<HoverPoint>>,
            Option<&TextUiBorder>,
            Option<&DisabledTextEffect>,
        ),
        Or<(Changed<FlexibleTextUiMultiline>, Changed<CalculatedSizeTty>)>,
    >,
) {
    for (
        text_ui,
        size,
        mut rendering,
        disabled,
        hover_point,
        text_ui_border,
        disabled_text_effect,
    ) in buttons.iter_mut()
    {
        if size.is_empty() {
            rendering.update(vec![]);
        }
        let borders_len = if text_ui_border.is_some() { 2 } else { 0 };
        let wrapped_desc = textwrap::wrap(text_ui.text.as_str(), size.width() - borders_len);
        let mut charmi = CharacterMapImage::new();
        let title_len = 0;
        // TODO title info, adjust the following as well
        for desc_line in wrapped_desc.into_iter().take(size.height() - title_len) {
            let row = charmi.new_row();
            // TODO add borders
            /*let render_text = text_ui.text.with_exact_width(text_len);
            let render_text = match text_ui_border {
                None => render_text,
                Some(TextUiBorder::Brackets) => format!("[{}]", render_text),
                Some(TextUiBorder::Parenthesis) => format!("[{}]", render_text),
            };*/
            row.add_text(desc_line, &text_ui.style);
        }
        if let (true, Some(effect)) = (disabled, disabled_text_effect) {
            charmi.apply_effect(effect);
        } else if matches!(hover_point, Some(Some(_))) {
            // Why not?
            charmi.apply_effect(&ContentStyle::new().reverse());
        }
        rendering.update_charmie(charmi)
    }
}

pub fn sys_tooltip_on_hover(
    tooltips: Query<(AsDeref<Tooltip>, AsDeref<HoverPoint>)>,
    mut tooltip_bars: Query<&mut FlexibleTextUi, With<TooltipBar>>,
) {
    let tooltip = tooltips
        .iter()
        .find_map(|(tooltip, hover_point)| hover_point.map(|_| tooltip.clone().into_owned()));
    for tooltip_bar_text_ui in tooltip_bars.iter_mut() {
        tooltip_bar_text_ui
            .map_unchanged(|ui| &mut ui.text)
            .set_if_neq(tooltip.clone().unwrap_or_default());
    }
}

pub fn sys_apply_hover(
    mut evr_mouse_tty: EventReader<MouseEventTty>,
    mut hoverable_ui: Query<(AsDerefMut<HoverPoint>,)>,
) {
    for event in evr_mouse_tty.read() {
        if let Ok((mut hover_point,)) = hoverable_ui.get_mut(event.entity()) {
            if !event.is_top_entity_or_ancestor() {
                hover_point.set_if_neq(None);
            } else if matches!(
                event.event_kind(),
                MouseEventTtyKind::Moved | MouseEventTtyKind::Drag { .. }
            ) {
                hover_point.set_if_neq(Some(event.relative_pos()));
            }
        }
    }
}
