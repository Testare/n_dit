use crate::term::layout::{CalculatedSizeTty, FitToSize, StyleTty};
use crate::term::node_ui::NodeUiQ;
use crate::term::node_ui::NodeFocus;
use crate::term::render::{RenderTtySet, UpdateRendering};
use crate::term::TerminalFocusMode;

use super::NodeUiQReadOnlyItem;
use bevy::app::{SystemAppConfig, SystemAppConfigs};
use bevy::ecs::query::WorldQuery;
use game_core::node::NodePiece;
use game_core::{prelude::*, Actions, Curio, Description, MaximumSize, Mon, MovementSpeed, Team};
use taffy::style::Dimension;

#[derive(WorldQuery)]
pub struct NodePieceQ {
    piece: &'static NodePiece,
    team: Option<&'static Team>,
    curio: Option<&'static Curio>,
    mon: Option<&'static Mon>,
    actions: Option<&'static Actions>,
    description: Option<&'static Description>,
    speed: Option<&'static MovementSpeed>,
    max_size: Option<&'static MaximumSize>,
}

pub trait NodeUi {
    fn style_update_system() -> Option<SystemAppConfig>;
    fn render_system() -> SystemAppConfig;
    fn ui_systems() -> SystemAppConfigs {
        let render_system = Self::render_system()
            .in_set(RenderTtySet::PostCalculateLayout)
            .in_set(OnUpdate(TerminalFocusMode::Node));
        if let Some(update_system) = Self::style_update_system() {
            (
                render_system,
                update_system
                    .in_set(RenderTtySet::PreCalculateLayout)
                    .in_set(OnUpdate(TerminalFocusMode::Node)),
            )
                .into_app_configs()
        } else {
            (render_system,).into_app_configs()
        }
    }
}

trait SimpleSubmenu {
    fn height(selected: &NodePieceQItem<'_>) -> Option<usize>;
    fn render(
        selected: &NodePieceQItem<'_>,
        size: &CalculatedSizeTty,
        node_data: &NodeUiQReadOnlyItem,
    ) -> Option<Vec<String>>;
}

#[derive(Component, Debug)]
pub struct MenuUiLabel;

#[derive(Component, Debug)]
pub struct MenuUiStats;

#[derive(Component, Debug)]
pub struct MenuUiActions;

#[derive(Component, Debug)]
pub struct MenuUiDescription;

impl SimpleSubmenu for MenuUiLabel {
    fn height(selected: &NodePieceQItem<'_>) -> Option<usize> {
        if selected.curio.is_some() {
            Some(3)
        } else {
            Some(2)
        }
    }

    fn render(
        selected: &NodePieceQItem<'_>,
        _size: &CalculatedSizeTty,
        _node_data: &NodeUiQReadOnlyItem,
    ) -> Option<Vec<String>> {
        let mut label = vec![selected.piece.display_id().clone()];

        if let Some(name) = selected
            .curio
            .map(Curio::name)
            .or_else(|| selected.mon.and(Some("Mon")))
            .map(str::to_owned)
        {
            label.push(name);
        }
        Some(label)
    }
}

impl SimpleSubmenu for MenuUiStats {
    fn height(selected: &NodePieceQItem<'_>) -> Option<usize> {
        let stats_to_display = if selected.max_size.is_some() { 1 } else { 0 }
            + if selected.speed.is_some() { 1 } else { 0 };
        if stats_to_display > 0 {
            Some(stats_to_display + 1)
        } else {
            None
        }
    }

    fn render(
        selected: &NodePieceQItem<'_>,
        size: &CalculatedSizeTty,
        node_data: &NodeUiQReadOnlyItem,
    ) -> Option<Vec<String>> {
        if selected.max_size.is_some() || selected.speed.is_some() {
            let mut stats = vec![format!("{0:-<1$}", "-Stats", size.width())];
            if let Some(max_size) = selected.max_size {
                let size = node_data.grid.len_of(
                    node_data
                        .selected_entity
                        .expect("stats display system should not run if no entity selected"),
                );
                stats.push(format!("Size:  {}/{}", size, **max_size));
            }
            if let Some(speed) = selected.speed {
                stats.push(format!("Speed: {}", **speed));
            }

            Some(stats)
        } else {
            None
        }
    }
}

impl SimpleSubmenu for MenuUiActions {
    fn height(selected: &NodePieceQItem<'_>) -> Option<usize> {
        let actions = selected.actions.as_deref()?;
        Some(actions.len() + 1)
    }

    fn render(
        selected: &NodePieceQItem<'_>,
        size: &CalculatedSizeTty,
        _node_data: &NodeUiQReadOnlyItem,
    ) -> Option<Vec<String>> {
        let actions = selected.actions.as_deref()?;
        let mut menu = vec![format!("{0:-<1$}", "-Actions", size.width())];
        for action in actions.iter() {
            menu.push(action.name.clone());
        }
        Some(menu)
    }
}

impl NodeUi for MenuUiDescription {
    fn style_update_system() -> Option<SystemAppConfig> {
        let mut last_nonzero_width: usize = 13; // TODO define constant for default width

        Some((move | node_data: Query<NodeUiQ>,
            node_focus: Res<NodeFocus>,
            node_pieces: Query<NodePieceQ>,
            mut ui: Query<(&mut StyleTty, &CalculatedSizeTty), With<MenuUiDescription>>
            | {
            

            if let Ok((mut style, size)) = ui.get_single_mut() {
                if size.width() != 0 {
                    last_nonzero_width = size.width();
                }
                let new_height = selected_piece_data(&node_data, node_focus, &node_pieces)
                    .and_then(|selected| {
                        // We use current size width here as an estimate, this might cause flickering if the menu changes width
                        Some(textwrap::wrap(selected.description?.as_str(), last_nonzero_width).len() as f32 + 1.0)
                    })
                    .unwrap_or(0.0);
                if Dimension::Points(new_height) != style.min_size.height {
                    style.min_size.height = Dimension::Points(new_height);
                    style.display = if new_height == 0.0 {
                        style.size.height = Dimension::Points(new_height);
                        taffy::style::Display::None
                    } else {
                        // Give a little extra for padding if we can
                        style.size.height = Dimension::Points(new_height);
                        taffy::style::Display::Flex
                    };
                }
            }
        }).into_app_config())
    }

    fn render_system() -> SystemAppConfig {
        (|node_data: Query<NodeUiQ>,
          node_focus: Res<NodeFocus>,
          node_pieces: Query<NodePieceQ>,
          mut commands: Commands,
          ui: Query<(Entity, &CalculatedSizeTty), With<MenuUiDescription>>| {
            if let Ok((id, size)) = ui.get_single() {
                let rendering = selected_piece_data(&node_data, node_focus, &node_pieces)
                    .and_then(|selected| {
                        let wrapped_desc =
                            textwrap::wrap(selected.description?.as_str(), size.width());
                        let mut menu = vec![format!("{0:-<1$}", "-Desc", size.width())];
                        for desc_line in wrapped_desc.into_iter() {
                            menu.push(desc_line.into_owned());
                        }
                        Some(menu)
                    })
                    .unwrap_or_default();

                commands
                    .entity(id)
                    .update_rendering(rendering.fit_to_size(size));
            }
        })
        .into_app_config()
    }
}

impl<S: SimpleSubmenu + Component> NodeUi for S {
    fn style_update_system() -> Option<SystemAppConfig> {
        Some(style_simple_submenu::<S>.into_app_config())
    }

    fn render_system() -> SystemAppConfig {
        render_simple_submenu::<S>.into_app_config()
    }
}

/// System for adjusting the height of a simple submenu
fn style_simple_submenu<T: SimpleSubmenu + Component>(
    node_data: Query<NodeUiQ>,
    node_focus: Res<NodeFocus>,
    node_pieces: Query<NodePieceQ>,
    mut ui: Query<&mut StyleTty, With<T>>,
) {
    if let Ok((mut style)) = ui.get_single_mut() {
        let new_height = selected_piece_data(&node_data, node_focus, &node_pieces)
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

/// System for rendering a simple submenu
fn render_simple_submenu<T: SimpleSubmenu + Component>(
    node_data: Query<NodeUiQ>,
    node_focus: Res<NodeFocus>,
    node_pieces: Query<NodePieceQ>,
    mut commands: Commands,
    ui: Query<(Entity, &CalculatedSizeTty), With<T>>,
) {
    if let Ok((id, size)) = ui.get_single() {
        let rendering = node_focus
            .and_then(|node_id| {
                let node_data = node_data.get(node_id).ok()?;
                let selected = node_pieces.get((**node_data.selected_entity)?).ok()?;
                T::render(&selected, &size, &node_data)
            })
            .unwrap_or_default();
        commands
            .entity(id)
            .update_rendering(rendering.fit_to_size(size));
    }
}

/// Helper method, since most submenus depend on getting the selected entity in the grid
fn selected_piece_data<'a>(
    node_data: &Query<NodeUiQ>,
    node_focus: Res<NodeFocus>,
    node_pieces: &'a Query<NodePieceQ>,
) -> Option<NodePieceQItem<'a>> {
    node_focus.and_then(|node_id| {
        let node_data = node_data.get(node_id).ok()?;
        let selected = node_pieces.get((**node_data.selected_entity)?).ok()?;
        Some(selected)
    })
}
