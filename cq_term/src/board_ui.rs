use charmi::CharacterMapImage;
use game_core::board::{BoardPiece, BoardPosition};

use crate::prelude::*;
use crate::render::{TerminalRendering, RENDER_TTY_SCHEDULE};

const BOARD_FILL_CHAR: Option<char> = Some('#');

#[derive(Debug, Default)]
pub struct BoardUiPlugin;

impl Plugin for BoardUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(RENDER_TTY_SCHEDULE, (sys_render_board,));
    }
}

#[derive(Clone, Component, Debug, Deref, DerefMut, Reflect)]
#[reflect(Component)]
pub struct BoardPieceUi(pub Entity); // track Board Piece

impl FromWorld for BoardPieceUi {
    fn from_world(world: &mut World) -> Self {
        Self(world.spawn_empty().id())
    }
}

#[derive(Clone, Component, Debug, Deref, DerefMut, Reflect)]
#[reflect(Component)]
pub struct BoardUi(pub Entity); // track Board

impl FromWorld for BoardUi {
    fn from_world(world: &mut World) -> Self {
        Self(world.spawn_empty().id())
    }
}

#[derive(Clone, Component, Debug, Default, Deref, DerefMut, Reflect)]
#[reflect(Component)]
pub struct BoardBackground(pub Handle<CharacterMapImage>);

fn sys_render_board(
    ast_charmi: Res<Assets<CharacterMapImage>>,
    board_pieces: Query<(
        AsDerefCopied<Parent>, // NOTE this means board pieces must be direct children of board.
        AsDerefCopied<BoardPosition>,
        &TerminalRendering,
    ), (With<BoardPiece>, Without<BoardUi>)>,
    mut board_uis: Query<(
        AsDeref<BoardBackground>,
        &mut TerminalRendering,
        AsDerefCopied<BoardUi>,
    )>,
) {
    for (background_handle, mut tr, board_id) in board_uis.iter_mut() {
        let charmi = ast_charmi.get(background_handle);
        if let Some(mut charmi) = charmi.cloned() {
            for (bp_board, UVec2{ x, y }, bp_tr) in board_pieces.iter() {
                if bp_board == board_id {
                    charmi = charmi.draw(bp_tr.charmie(), x, y, BOARD_FILL_CHAR);
                }
            }
            tr.update_charmie(charmi);
        }
    }
}
