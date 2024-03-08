use bevy::ecs::component::Component;
use charmi::CharacterMapImage;
use game_core::board::SimplePieceInfo;
use game_core::player::{ForPlayer, Player};

use super::{BoardPieceUi, SelectedBoardPieceUi};
use crate::layout::CalculatedSizeTty;
use crate::prelude::*;
use crate::render::TerminalRendering;

#[derive(Debug)]
pub struct SidebarPlugin;

impl Plugin for SidebarPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, sys_render_info_panel);
    }
}

#[derive(Component, Debug, Default)]
pub struct InfoPanel;

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
