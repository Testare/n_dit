pub mod context_menu;
mod input_actions;
mod popup;
mod select_menu;

use std::borrow::{Borrow, Cow};

use bevy::ecs::query::Has;
use charmi::CharacterMapImage;
use crossterm::style::{ContentStyle, Stylize};
use pad::PadStr;
use taffy::prelude::Size;
use taffy::style::Dimension;

use self::context_menu::ContextMenuPlugin;
pub use self::popup::*;
use crate::input_event::{
    MouseEventListener, MouseEventTty, MouseEventTtyDisabled, MouseEventTtyKind,
    MouseLastPositionTty,
};
use crate::layout::{CalculatedSizeTty, GlobalTranslationTty, StyleTty, VisibilityTty};
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

#[derive(Debug, Default)]
pub struct BaseUiPlugin;

impl Plugin for BaseUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            RENDER_TTY_SCHEDULE,
            (
                sys_apply_hover,
                sys_render_flexible_text.in_set(RenderTtySet::RenderLayouts),
                popup::sys_render_popup_menu.in_set(RenderTtySet::RenderLayouts),
                sys_tooltip_on_hover,
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

#[derive(Component, Debug, Reflect)]
pub enum TextUiBorder {
    Brackets,
    Parenthesis,
}

#[derive(Clone, Component, Debug, Deref, DerefMut)]
pub struct DisabledTextEffect(ContentStyle);

// TODO HoverTextEffect when mouse events supports it

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
    res_mouse_last_pos: Res<MouseLastPositionTty>,
    mut evr_mouse_tty: EventReader<MouseEventTty>,
    changed_visibility: Query<
        (
            Entity,
            OrBool<AsDerefCopied<VisibilityTty>, true>,
            AsDeref<CalculatedSizeTty>,
            AsDeref<GlobalTranslationTty>,
        ),
        (
            Or<(Changed<VisibilityTty>, Changed<CalculatedSizeTty>)>,
            With<HoverPoint>,
        ),
    >,
    new_disabled: Query<Entity, (With<HoverPoint>, Added<MouseEventTtyDisabled>)>,
    mut hoverable_ui: Query<(AsDerefMut<HoverPoint>,)>,
) {
    for event in evr_mouse_tty.read() {
        if let Ok((mut hover_point,)) = hoverable_ui.get_mut(event.entity()) {
            if !event.is_top_entity_or_ancestor() {
                hover_point.set_if_neq(None);
            } else if matches!(event.event_kind(), MouseEventTtyKind::Moved) {
                hover_point.set_if_neq(Some(event.relative_pos()));
            }
        }
    }
    for (id, is_visible, size, global_translation) in changed_visibility.iter() {
        if !is_visible || size.x == 0 || size.y == 0 {
            if let Ok((mut hover_point,)) = hoverable_ui.get_mut(id) {
                hover_point.set_if_neq(None);
            }
        } else {
            // TODO adjust this to take render order into account, otherwise if an item
            // that changes size underneath another could get its own duplicate hoverpoint,
            // even without being a parent entity
            let relative_pos_x = res_mouse_last_pos
                .x
                .checked_sub(global_translation.x)
                .filter(|x| *x < size.x);
            let relative_pos_y = res_mouse_last_pos
                .y
                .checked_sub(global_translation.y)
                .filter(|y| *y < size.y);
            if let Some((x, y)) = relative_pos_x.zip(relative_pos_y) {
                if let Ok((mut hover_point,)) = hoverable_ui.get_mut(id) {
                    hover_point.set_if_neq(Some(UVec2 { x, y }));
                }
            }
        }
    }
    for id in new_disabled.iter() {
        if let Ok((mut hover_point,)) = hoverable_ui.get_mut(id) {
            hover_point.set_if_neq(None);
        }
    }
}
