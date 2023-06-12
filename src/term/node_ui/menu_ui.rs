mod actions;
mod card_selection;

pub use actions::MenuUiActions;
use bevy::app::{SystemAppConfig, SystemAppConfigs};
use bevy::ecs::query::WorldQuery;
use bevy::ecs::system::{StaticSystemParam, SystemParam};
pub use card_selection::MenuUiCardSelection;
use game_core::node::NodePiece;
use game_core::prelude::*;
use game_core::{
    AccessPoint, Actions, Curio, Description, IsTapped, MaximumSize, MovementSpeed, MovesTaken,
    Pickup, Team,
};
use taffy::style::Dimension;

use super::registry::GlyphRegistry;
use super::NodeUiDataParam;
use crate::term::layout::{CalculatedSizeTty, FitToSize, StyleTty};
use crate::term::node_ui::{NodeFocus, NodeUiQ};
use crate::term::render::{RenderTtySet, UpdateRendering};
use crate::term::TerminalFocusMode;

#[derive(WorldQuery)]
pub struct NodePieceQ {
    piece: &'static NodePiece,
    team: Option<&'static Team>,
    curio: Option<&'static Curio>,
    pickup: Option<&'static Pickup>,
    actions: Option<&'static Actions>,
    description: Option<&'static Description>,
    speed: Option<&'static MovementSpeed>,
    max_size: Option<&'static MaximumSize>,
    moves_taken: Option<&'static MovesTaken>,
    is_tapped: Option<&'static IsTapped>,
    access_point: Option<&'static AccessPoint>,
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
    type RenderSystemParam: SystemParam;

    fn height(selected: &NodePieceQItem<'_>) -> Option<usize>;
    fn render<'w, 's>(
        selected: &NodePieceQItem<'_>,
        size: &CalculatedSizeTty,
        sys_param: <Self::RenderSystemParam as SystemParam>::Item<'w, 's>,
    ) -> Option<Vec<String>>;
}

#[derive(Component, Debug)]
pub struct MenuUiLabel;

#[derive(Component, Debug)]
pub struct MenuUiStats;

#[derive(Component, Debug, Default)]
pub struct MenuUiDescription;

impl SimpleSubmenu for MenuUiLabel {
    type RenderSystemParam = Res<'static, GlyphRegistry>;
    fn height(_: &NodePieceQItem<'_>) -> Option<usize> {
        Some(2)
    }

    fn render<'w, 's>(
        selected: &NodePieceQItem<'_>,
        _size: &CalculatedSizeTty,
        glyph_registry: <Self::RenderSystemParam as SystemParam>::Item<'w, 's>,
    ) -> Option<Vec<String>> {
        let display_name = selected.piece.display_id();
        let glyph = (**glyph_registry)
            .get(display_name)
            .map(|s| s.as_str())
            .unwrap_or("??");

        let is_tapped = selected
            .is_tapped
            .map(|is_tapped| **is_tapped)
            .unwrap_or(false);

        let mut label = vec![format!(
            "[{}]{}",
            glyph,
            if is_tapped { " (tapped)" } else { "" }
        )];
        if selected.access_point.is_some() {
            label.push("Access Point".to_owned());
        } else if let Some(name) = selected
            .curio
            .map(Curio::name)
            .or_else(|| {
                selected.pickup.map(|pickup| match pickup {
                    Pickup::Mon(_) => "Mon",
                    Pickup::Card(_) => "Card: ??",
                    Pickup::Item(_) => "Item: ??",
                })
            })
            .map(str::to_owned)
        {
            label.push(name);
        }
        Some(label)
    }
}
impl SimpleSubmenu for MenuUiStats {
    type RenderSystemParam = NodeUiDataParam<'static, 'static>;

    fn height(selected: &NodePieceQItem<'_>) -> Option<usize> {
        let stats_to_display = if selected.max_size.is_some() { 1 } else { 0 }
            + if selected.speed.is_some() { 1 } else { 0 };
        if stats_to_display > 0 {
            Some(stats_to_display + 1)
        } else {
            None
        }
    }

    fn render<'w, 's>(
        selected: &NodePieceQItem<'_>,
        size: &CalculatedSizeTty,
        node_ui_data: <Self::RenderSystemParam as SystemParam>::Item<'w, 's>,
    ) -> Option<Vec<String>> {
        if selected.max_size.is_some() || selected.speed.is_some() {
            let mut stats = vec![format!("{0:─<1$}", "─Stats", size.width())];
            if let Some(max_size) = selected.max_size {
                let node_data = node_ui_data.node_data()?;
                let size = node_data.grid.len_of(
                    node_data
                        .selected_entity
                        .expect("stats display system should not run if no entity selected"),
                );
                stats.push(format!("Size:  {}/{}", size, **max_size));
            }
            /*if let Some(speed) = selected.speed {
                stats.push(format!("Speed: {}", **speed));
            }*/
            if let Some(speed) = selected.speed {
                let moves_taken = selected
                    .moves_taken
                    .map(|moves_taken| **moves_taken)
                    .unwrap_or(0);
                stats.push(format!("Moves: {}/{}", moves_taken, **speed));
            }

            Some(stats)
        } else {
            None
        }
    }
}

impl MenuUiDescription {
    fn style_update_system(
        node_data: Query<NodeUiQ>,
        node_focus: Res<NodeFocus>,
        node_pieces: Query<NodePieceQ>,
        mut ui: Query<(&mut StyleTty, &CalculatedSizeTty), With<MenuUiDescription>>,
        mut last_nonzero_width: Local<usize>,
    ) {
        if let Ok((mut style, size)) = ui.get_single_mut() {
            if size.width() != 0 {
                *last_nonzero_width = size.width();
            }
            let new_height = selected_piece_data(&node_data, node_focus, &node_pieces)
                .and_then(|selected| {
                    // We use current size width here as an estimate, this might cause flickering if the menu changes width
                    Some(
                        textwrap::wrap(selected.description?.as_str(), *last_nonzero_width).len()
                            as f32
                            + 1.0,
                    )
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
    }

    fn render_system(
        node_data: Query<NodeUiQ>,
        node_focus: Res<NodeFocus>,
        node_pieces: Query<NodePieceQ>,
        mut commands: Commands,
        ui: Query<(Entity, &CalculatedSizeTty), With<MenuUiDescription>>,
    ) {
        if let Ok((id, size)) = ui.get_single() {
            let rendering = selected_piece_data(&node_data, node_focus, &node_pieces)
                .and_then(|selected| {
                    let wrapped_desc = textwrap::wrap(selected.description?.as_str(), size.width());
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
    }
}

impl NodeUi for MenuUiDescription {
    fn style_update_system() -> Option<SystemAppConfig> {
        Some(Self::style_update_system.into_app_config())
    }

    fn render_system() -> SystemAppConfig {
        Self::render_system.into_app_config()
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
    if let Ok(mut style) = ui.get_single_mut() {
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
fn render_simple_submenu<'w, 's, T: SimpleSubmenu + Component>(
    node_data: Query<NodeUiQ>,
    node_focus: Res<NodeFocus>,
    node_pieces: Query<NodePieceQ>,
    mut commands: Commands,
    ui: Query<(Entity, &CalculatedSizeTty), With<T>>,
    render_param: StaticSystemParam<'w, 's, T::RenderSystemParam>,
) {
    if let Ok((id, size)) = ui.get_single() {
        let rendering = node_focus
            .and_then(|node_id| {
                let node_data = node_data.get(node_id).ok()?;
                let selected = node_pieces.get((**node_data.selected_entity)?).ok()?;
                T::render(&selected, &size, render_param.into_inner())
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
