use bevy::ecs::component::Component;
use bevy::hierarchy::DespawnRecursiveExt;
use charmi::CharacterMapImage;
use crossterm::style::{ContentStyle, Stylize};
use game_core::board::SimplePieceInfo;
use game_core::player::{ForPlayer, Player};

use super::{BoardPieceUi, SelectedBoardPieceUi};
use crate::base_ui::context_menu::{
    ContextAction, ContextActionDelegate, ContextActions, ContextActionsInteraction,
};
use crate::base_ui::ButtonUiBundle;
use crate::layout::{CalculatedSizeTty, StyleTty};
use crate::prelude::*;
use crate::render::{RenderTtySet, TerminalRendering};

#[derive(Debug)]
pub struct SidebarPlugin;

#[derive(Component, Debug)]
pub struct ActionsPanelIgnoredAction;

impl Plugin for SidebarPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                (sys_update_context_action_panel, sys_update_info_panel)
                    .in_set(RenderTtySet::PreCalculateLayout),
                sys_render_info_panel.in_set(RenderTtySet::PostCalculateLayout), // RenderLayouts?
            ),
        );
    }
}

#[derive(Component, Debug, Default)]
pub struct InfoPanel;

#[derive(Component, Debug, Default)]
pub struct ActionsPanel;

/// TODO Cache data from this in the render panel
/// TODO de-dupe this logic with node description ui
pub fn sys_update_info_panel(
    q_player: Query<AsDerefCopied<SelectedBoardPieceUi>, With<Player>>,
    q_board_piece_ui: Query<AsDerefCopied<BoardPieceUi>>,
    q_board_piece: Query<AsDeref<SimplePieceInfo>>,
    mut q_info_panel: Query<
        (AsDerefCopied<ForPlayer>, &CalculatedSizeTty, &mut StyleTty),
        With<InfoPanel>,
    >,
) {
    for (player_id, size, mut style) in q_info_panel.iter_mut() {
        let info = get_assert!(player_id, q_player, |selected_board_piece| {
            let bp_ui_id = selected_board_piece?;
            let bp_id = q_board_piece_ui.get(bp_ui_id).ok()?;
            q_board_piece.get(bp_id).ok()
        });
        let width = size.width();
        let height = taffy::prelude::length(if let Some(info) = info {
            textwrap::wrap(info, width).len() + 1
        } else {
            0
        } as f32);
        if style.size.height != height {
            style.size.height = height;
        }
    }
}

pub fn sys_render_info_panel(
    q_player: Query<AsDerefCopied<SelectedBoardPieceUi>, With<Player>>,
    q_board_piece_ui: Query<AsDerefCopied<BoardPieceUi>>,
    q_board_piece: Query<AsDeref<SimplePieceInfo>>,
    mut q_info_panel: Query<
        (
            AsDerefCopied<ForPlayer>,
            &CalculatedSizeTty,
            &mut TerminalRendering,
        ),
        With<InfoPanel>,
    >,
) {
    for (player_id, size, mut tr) in q_info_panel.iter_mut() {
        let info = get_assert!(player_id, q_player, |selected_board_piece| {
            let bp_ui_id = selected_board_piece?;
            let bp_id = q_board_piece_ui.get(bp_ui_id).ok()?;
            q_board_piece.get(bp_id).ok()
        });
        if let Some(info) = info {
            // TODO consider why I'm duplicating the work in node_ui::menu_ui::description
            let width = size.width();
            let title = format!("{0:─<1$}", "─Info", width);
            let wrapped_desc: CharacterMapImage = std::iter::once(title)
                .chain(
                    textwrap::wrap(info, width)
                        .into_iter()
                        .map(|s| s.into_owned()),
                )
                .collect();
            tr.update_charmie(wrapped_desc)
        } else {
            tr.update(vec![]);
        }
    }
}

pub fn sys_update_context_action_panel(
    mut commands: Commands,
    mut q_actions_panel: Query<(Entity, &ForPlayer, &mut StyleTty), With<ActionsPanel>>,
    q_player: Query<(Entity, Ref<SelectedBoardPieceUi>), With<Player>>,
    q_context_action: Query<&ContextAction, Without<ActionsPanelIgnoredAction>>,
    q_board_piece_ui: Query<Ref<ContextActions>, With<BoardPieceUi>>,
) {
    for (player_id, selected_board_piece_ui) in q_player.iter() {
        if let Some((panel_id, _, mut style)) = ForPlayer::get_mut(&mut q_actions_panel, player_id)
        {
            if let Some((board_piece_ca, board_piece_id)) =
                selected_board_piece_ui.and_then(|id| q_board_piece_ui.get(id).ok().zip(Some(id)))
            {
                if !selected_board_piece_ui.is_changed() && !board_piece_ca.is_changed() {
                    continue;
                }
                let mut height = 0.0;
                commands
                    .entity(panel_id)
                    .despawn_descendants()
                    .with_children(|panel| {
                        for ca_id in board_piece_ca.actions() {
                            // TODO add marker component to filter out certain actions
                            if let Ok(ca) = q_context_action.get(ca_id) {
                                panel.spawn((
                                    ButtonUiBundle::new(ca.name(), ContentStyle::new().yellow()),
                                    ContextActions::new(player_id, &[ca_id]),
                                    ContextActionDelegate::new(board_piece_id),
                                    ContextActionsInteraction::SingleActionOnly,
                                ));
                                height += 1.0;
                            }
                        }
                    });

                style.size.height = taffy::prelude::length(height);

                // TODO create new children
            } else if selected_board_piece_ui.is_changed() {
                commands.entity(panel_id).despawn_descendants();
            }
        }
    }
}
