use std::marker::PhantomData;

use bevy::ecs::system::StaticSystemParam;
use game_core::player::{ForPlayer, Player};
use game_core::NDitCoreSet;
use taffy::style::Dimension;

use super::{NodePieceQ, NodeUi, SelectedEntity, SimpleSubmenu};
use crate::term::layout::{CalculatedSizeTty, FitToSize, StyleTty};
use crate::term::prelude::*;
use crate::term::render::{RenderTtySet, UpdateRendering};
use crate::term::TerminalFocusMode;

pub struct SimpleSubMenuPlugin<S> {
    _marker: PhantomData<S>,
}

impl<S: SimpleSubmenu + Component + Sync + Send + 'static> Plugin for SimpleSubMenuPlugin<S> {
    fn build(&self, app: &mut App) {
        app.add_systems(
            (
                style_simple_submenu::<S>.in_set(RenderTtySet::PreCalculateLayout),
                render_simple_submenu::<S>.in_set(RenderTtySet::PostCalculateLayout),
            )
                .in_set(OnUpdate(TerminalFocusMode::Node)),
        );
        if let Some(layout_system) = S::layout_event_system() {
            app.add_system(layout_system.in_set(NDitCoreSet::ProcessInputs));
        }
    }
}

impl<S: SimpleSubmenu + Component> NodeUi for S {
    type UiBundle = ();
    type UiPlugin = SimpleSubMenuPlugin<S>;

    fn ui_bundle() -> Self::UiBundle {
        ()
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
    players: Query<&SelectedEntity, With<Player>>,
    mut ui: Query<(&mut StyleTty, &ForPlayer), With<T>>,
) {
    for (mut style, ForPlayer(player)) in ui.iter_mut() {
        if let Ok(selected_entity) = players.get(*player) {
            let new_height = selected_entity
                .of(&node_pieces)
                .and_then(|selected| Some(T::height(&selected)? as f32))
                .unwrap_or(0.0);

            if Dimension::Points(new_height) != style.min_size.height {
                style.min_size.height = Dimension::Points(new_height);
                style.display = if new_height == 0.0 {
                    style.size.height = Dimension::Points(new_height);
                    taffy::style::Display::None
                } else {
                    // Give a little extra for padding if we can
                    style.size.height = Dimension::Points(new_height + 1.0);
                    taffy::style::Display::Flex
                };
            }
        }
    }
}

/// System for rendering a simple submenu
fn render_simple_submenu<'w, 's, T: SimpleSubmenu + Component>(
    mut commands: Commands,
    node_pieces: Query<NodePieceQ>,
    players: Query<&SelectedEntity, With<Player>>,
    ui: Query<(Entity, &CalculatedSizeTty, &ForPlayer), With<T>>,
    render_param: StaticSystemParam<'w, 's, T::RenderSystemParam>,
) {
    let render_param = render_param.into_inner();
    for (id, size, ForPlayer(player)) in ui.iter() {
        if let Ok(selected_entity) = players.get(*player) {
            let rendering = selected_entity
                .of(&node_pieces)
                .and_then(|selected| T::render(*player, &selected, &size, &render_param))
                .unwrap_or_default();
            commands
                .entity(id)
                .update_rendering(rendering.fit_to_size(size));
        }
    }
}
