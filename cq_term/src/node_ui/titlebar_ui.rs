use game_core::prelude::*;

use super::{NodeUi, NodeUiQItem};
use crate::layout::StyleTty;
use crate::render::{RenderTtySet, TerminalRendering, RENDER_TTY_SCHEDULE};

#[derive(Component, Default)]
pub struct TitleBarUi;

pub fn render_title_bar_system(
    mut render_title_bar: Query<&mut TerminalRendering, With<TitleBarUi>>,
) {
    let rendered_text = vec!["Common Quest".to_owned()];
    for mut tr in render_title_bar.iter_mut() {
        tr.update(rendered_text.clone());
    }
}

impl Plugin for TitleBarUi {
    fn build(&self, app: &mut App) {
        app.add_systems(
            RENDER_TTY_SCHEDULE,
            render_title_bar_system.in_set(RenderTtySet::PostCalculateLayout),
        );
    }
}

impl NodeUi for TitleBarUi {
    const NAME: &'static str = "Node Title Bar";
    type UiBundleExtras = ();
    type UiPlugin = Self;

    fn initial_style(_: &NodeUiQItem) -> StyleTty {
        use taffy::prelude::*;

        StyleTty(Style {
            size: Size {
                width: Dimension::Auto,
                height: Dimension::Points(2.),
            },
            padding: Rect {
                bottom: LengthPercentage::Points(1.0),
                ..TaffyZero::ZERO
            },
            max_size: Size {
                width: Dimension::Points(100.0),
                height: Dimension::Auto,
            },
            flex_shrink: 0.0,
            ..default()
        })
    }

    fn ui_bundle_extras() -> Self::UiBundleExtras {}
}
