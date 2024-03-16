use std::marker::PhantomData;

use bevy::ecs::system::StaticSystemParam;
use game_core::player::{ForPlayer, Player};

use super::{NodePieceQ, NodeUi, SelectedNodePiece, SimpleSubmenu};
use crate::layout::{CalculatedSizeTty, StyleTty};
use crate::node_ui::NodeUiQItem;
use crate::prelude::*;
use crate::render::{RenderTtySet, TerminalRendering, RENDER_TTY_SCHEDULE};

#[derive(Debug)]
pub struct SimpleSubMenuPlugin<S> {
    _marker: PhantomData<S>,
}

impl<S: SimpleSubmenu + Component + Sync + Send + 'static> Plugin for SimpleSubMenuPlugin<S> {
    fn build(&self, app: &mut App) {
        app.add_systems(
            RENDER_TTY_SCHEDULE,
            (
                style_simple_submenu::<S>.in_set(RenderTtySet::AdjustLayoutStyle),
                render_simple_submenu::<S>.in_set(RenderTtySet::PostCalculateLayout),
            ),
        );
    }
}

impl<S: SimpleSubmenu + Component + Default> NodeUi for S {
    const NAME: &'static str = S::NAME;
    type UiBundleExtras = <S as SimpleSubmenu>::UiBundleExtras;
    type UiPlugin = SimpleSubMenuPlugin<S>;

    fn initial_style(_: &NodeUiQItem) -> StyleTty {
        <S as SimpleSubmenu>::initial_style()
    }

    fn ui_bundle_extras() -> Self::UiBundleExtras {
        <S as SimpleSubmenu>::ui_bundle_extras()
    }
}

impl<S> Default for SimpleSubMenuPlugin<S> {
    fn default() -> Self {
        SimpleSubMenuPlugin {
            _marker: PhantomData::<S>,
        }
    }
}

/// System for adjusting the height of a simple submenu
fn style_simple_submenu<T: SimpleSubmenu + Component>(
    node_pieces: Query<NodePieceQ>,
    players: Query<&SelectedNodePiece, With<Player>>,
    mut ui: Query<(&mut StyleTty, &ForPlayer), With<T>>,
) {
    use taffy::prelude::*;
    for (mut style, ForPlayer(player)) in ui.iter_mut() {
        if let Ok(selected_entity) = players.get(*player) {
            let new_height = selected_entity
                .of(&node_pieces)
                .and_then(|selected| Some(T::height(&selected)? as f32))
                .unwrap_or(0.0);

            if Dimension::Length(new_height) != style.min_size.height {
                style.min_size.height = length(new_height);
                style.display = if new_height == 0.0 {
                    style.size.height = length(new_height);
                    taffy::style::Display::None
                } else {
                    // Give a little extra for padding if we can
                    style.size.height = length(new_height + 1.0);
                    taffy::style::Display::Flex
                };
            }
        }
    }
}

/// System for rendering a simple submenu
fn render_simple_submenu<T: SimpleSubmenu + Component>(
    node_pieces: Query<NodePieceQ>,
    players: Query<&SelectedNodePiece, With<Player>>,
    mut uis: Query<(&CalculatedSizeTty, &ForPlayer, &mut TerminalRendering), With<T>>,
    render_param: StaticSystemParam<T::RenderSystemParam>,
) {
    let render_param = render_param.into_inner();
    for (size, ForPlayer(player), mut tr) in uis.iter_mut() {
        if let Ok(selected_entity) = players.get(*player) {
            let mut rendering = selected_entity
                .of(&node_pieces)
                .and_then(|selected| T::render(*player, &selected, size, &render_param))
                .unwrap_or_default();
            rendering.fit_to_size(size.x, size.y);
            tr.update_charmie(rendering);
        }
    }
}
