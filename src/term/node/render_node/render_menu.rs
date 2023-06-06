use crate::term::TerminalFocusMode;
use crate::term::layout::{CalculatedSizeTty, FitToSize, StyleTty};
use crate::term::node::render_node::RenderNodeData;
use crate::term::node::NodeFocus;
use crate::term::render::{UpdateRendering, RenderTtySet};

use super::RenderNodeDataReadOnlyItem;
use bevy::app::{SystemAppConfig, SystemAppConfigs};
use bevy::ecs::query::WorldQuery;
use game_core::node::NodePiece;
use game_core::{prelude::*, Actions, Curio, Description, MaximumSize, Mon, MovementSpeed, Team};
use pad::PadStr;
use taffy::style::Dimension;

#[derive(WorldQuery)]
pub struct NodePieceMenuData {
    piece: &'static NodePiece,
    team: Option<&'static Team>,
    curio: Option<&'static Curio>,
    mon: Option<&'static Mon>,
    actions: Option<&'static Actions>,
    description: Option<&'static Description>,
    speed: Option<&'static MovementSpeed>,
    max_size: Option<&'static MaximumSize>,
}



#[derive(Component, Debug)]
pub struct MenuUiLabel;

#[derive(Component, Debug)]
pub struct MenuUiStats;

#[derive(Component, Debug)]
pub struct MenuUiActions;

#[derive(Component, Debug)]
pub struct MenuUiDescription;

pub fn render_menu(
    node_render_data: &RenderNodeDataReadOnlyItem,
    node_pieces: &Query<NodePieceMenuData>,
    size: &CalculatedSizeTty,
) -> Vec<String> {
    if let Some(selected_entity) = node_render_data
        .grid
        .item_at(**node_render_data.node_cursor)
    {
        let selected = node_pieces
            .get(selected_entity)
            .expect("entities in entity grid should have NodePiece components");

        let mut unbound_vec = vec![
            selected.piece.display_id().clone(),
            // selected.team.map(|team| format!("Team: {:?}", team)),
            // selected.team.map(|_| "".to_owned()),
        ];

        if let Some(name) = selected
            .curio
            .map(Curio::name)
            .or_else(|| selected.mon.and(Some("Mon")))
            .map(str::to_owned)
        {
            unbound_vec.push(name);
        }

        if selected.max_size.is_some() || selected.speed.is_some() {
            unbound_vec.push(format!("{0:-<1$}", "-Stats", size.width()));
            if let Some(max_size) = selected.max_size {
                let size = node_render_data.grid.len_of(selected_entity);
                unbound_vec.push(format!("Size:  {}/{}", size, **max_size));
            }
            if let Some(speed) = selected.speed {
                unbound_vec.push(format!("Speed: {}", **speed));
            }
        }
        if let Some(actions) = selected.actions {
            unbound_vec.push(format!("{0:-<1$}", "-Actions", size.width()));
            for action in actions.iter() {
                // Record position of action
                unbound_vec.push(action.name.clone());
            }
        }
        if let Some(description) = selected.description {
            unbound_vec.push(format!("{0:-<1$}", "-Desc", size.width()));
            let wrapped_desc = textwrap::wrap(description.as_str(), size.width());
            for desc_line in wrapped_desc.into_iter() {
                unbound_vec.push(desc_line.into_owned());
            }
        }

        /* unbound_vec
        .into_iter()
        .map(|line| (line.with_exact_width(size.width())))
        .take(size.height())
        .collect()
        */
        unbound_vec.fit_to_size(size)
    } else {
        vec![]
    }
    // Get node cursor
    // Get Entity from that position
    // Determine if it is a curio (friendly or not), pickup, or access point
}

pub fn calculate_action_menu_style(
    node_data: Query<RenderNodeData>,
    node_focus: Res<NodeFocus>,
    node_pieces: Query<NodePieceMenuData>,
    mut ui: Query<&mut StyleTty, With<MenuUiActions>>,
) {
    if let Ok(mut style) = ui.get_single_mut() {
        let new_height = selected_piece_data(&node_data, node_focus, &node_pieces)
            .and_then(|selected| {
                let actions = selected.actions.as_deref()?;
                Some(Dimension::Points(actions.len() as f32 + 1.0))
            })
            .unwrap_or(Dimension::Points(0.0));
        if style.size.height != new_height {
            style.size.height = new_height;
            style.display = if new_height == Dimension::Points(0.0) {
                taffy::style::Display::None
            } else {
                taffy::style::Display::Flex
            };
        }
    }
}

pub trait Submenu {
    // Render System
    // Optional height system
    // get systems
    fn get_style_update_system() -> Option<SystemAppConfig>;
    fn get_render_system() -> SystemAppConfig;
    fn get_systems() -> SystemAppConfigs {
        let render_system = Self::get_render_system().in_set(RenderTtySet::PostCalculateLayout).in_set(OnUpdate(TerminalFocusMode::Node));
        if let Some(update_system) = Self::get_style_update_system() {
            (
                render_system,
                update_system.in_set(RenderTtySet::PreCalculateLayout).in_set(OnUpdate(TerminalFocusMode::Node))
            ).into_app_configs()
        } else {
            (render_system,).into_app_configs()
        }
    }
}

trait SimpleSubmenu {
    fn height(selected: &NodePieceMenuDataItem<'_>) -> Option<usize>;
    fn render(selected: &NodePieceMenuDataItem<'_>, size: &CalculatedSizeTty) -> Option<Vec<String>>;
}

impl <S: SimpleSubmenu + Component> Submenu for S {
    fn get_style_update_system() -> Option<SystemAppConfig> {
        Some(style_simple_submenu::<S>.into_app_config())
    }

    fn get_render_system() -> SystemAppConfig {
        render_simple_submenu::<S>.into_app_config()
    }
}

impl SimpleSubmenu for MenuUiActions {
    fn height<'a>(selected: &NodePieceMenuDataItem<'_>) -> Option<usize> {
        let actions = selected.actions.as_deref()?;
        Some(actions.len() + 1)
    }

    fn render(selected: &NodePieceMenuDataItem<'_>, size: &CalculatedSizeTty) -> Option<Vec<String>> {
        let actions = selected.actions.as_deref()?;
        let mut menu = vec![format!("{0:-<1$}", "-Actions", size.width())];
        for action in actions.iter() {
            menu.push(action.name.clone());
        }
        Some(menu)
    }

}

fn style_simple_submenu<T: SimpleSubmenu + Component>(
    node_data: Query<RenderNodeData>,
    node_focus: Res<NodeFocus>,
    node_pieces: Query<NodePieceMenuData>,
    mut ui: Query<&mut StyleTty, With<MenuUiActions>>,
) {
    if let Ok((mut style)) = ui.get_single_mut() {
        let new_height = selected_piece_data(&node_data, node_focus, &node_pieces)
            .and_then(|selected| {
                Some(Dimension::Points(T::height(&selected)? as f32))
            })
            .unwrap_or(Dimension::Points(0.0));
        if style.size.height != new_height {
            style.size.height = new_height;
            style.display = if new_height == Dimension::Points(0.0) {
                taffy::style::Display::None
            } else {
                taffy::style::Display::Flex
            };
        }
    }

}


fn render_simple_submenu<T: SimpleSubmenu + Component>(
    node_data: Query<RenderNodeData>,
    node_focus: Res<NodeFocus>,
    node_pieces: Query<NodePieceMenuData>,
    mut commands: Commands,
    ui: Query<(Entity, &CalculatedSizeTty), With<T>>,
) {
    if let Ok((id, size)) = ui.get_single() {
        let rendering = selected_piece_data(&node_data, node_focus, &node_pieces)
            .and_then(|selected| {
                T::render(&selected, &size)
            })
            .unwrap_or_default();
        commands
            .entity(id)
            .update_rendering(rendering.fit_to_size(size));
    }
}

/*
pub fn action_menu(
    node_data: Query<RenderNodeData>,
    node_focus: Res<NodeFocus>,
    node_pieces: Query<NodePieceMenuData>,
    mut commands: Commands,
    ui: Query<(Entity, &CalculatedSizeTty), With<MenuUiActions>>,
) {
    if let Ok((id, size)) = ui.get_single() {
        let rendering = selected_piece_data(&node_data, node_focus, &node_pieces)
            .and_then(|selected| {
                let actions = selected.actions.as_deref()?;
                let mut menu = vec![format!("{0:-<1$}", "-Actions", size.width())];
                for action in actions.iter() {
                    menu.push(action.name.clone());
                }
                Some(menu)
            })
            .unwrap_or_default();

        commands
            .entity(id)
            .update_rendering(rendering.fit_to_size(size));
    }
}
*/

pub fn description(
    node_data: Query<RenderNodeData>,
    node_focus: Res<NodeFocus>,
    node_pieces: Query<NodePieceMenuData>,
    mut commands: Commands,
    ui: Query<(Entity, &CalculatedSizeTty), With<MenuUiDescription>>,
) {
    if let Ok((id, size)) = ui.get_single() {
        let rendering = selected_piece_data(&node_data, node_focus, &node_pieces).and_then(|selected| {
                let wrapped_desc = textwrap::wrap(selected.description?.as_str(), size.width());
                let mut menu = vec![format!("{0:-<1$}", "-Desc", size.width())];
                for desc_line in wrapped_desc.into_iter() {
                    menu.push(desc_line.into_owned());
                }
                Some(menu)
        }).unwrap_or_default();

        commands
            .entity(id)
            .update_rendering(rendering.fit_to_size(size));
    }
}

pub fn selected_piece_data<'a>(
    node_data: &Query<RenderNodeData>,
    node_focus: Res<NodeFocus>,
    node_pieces: &'a Query<NodePieceMenuData>,
) -> Option<NodePieceMenuDataItem<'a>> {
        node_focus
            .and_then(|node_id| {
                let node_data = node_data.get(node_id).ok()?;
                let selected = node_pieces.get((**node_data.selected_entity)?).ok()?;
                Some(selected)
            })
}