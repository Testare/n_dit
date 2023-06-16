use game_core::node::{InNode, Node};
use game_core::player::{ForPlayer, Player};

use super::{GridUi, NodeCursor};
use crate::term::layout::CalculatedSizeTty;
use crate::term::prelude::*;

#[derive(Component, Debug, Default, Deref, DerefMut, FromReflect, Reflect)]
pub struct Scroll2D(pub UVec2);

pub fn adjust_scroll(
    players: Query<(&NodeCursor, &InNode), With<Player>>,
    node_grids: Query<&EntityGrid, With<Node>>,
    mut ui: Query<(&CalculatedSizeTty, &mut Scroll2D, &ForPlayer), With<GridUi>>,
) {
    for (size, mut scroll, ForPlayer(player)) in ui.iter_mut() {
        if let Ok((cursor, InNode(node))) = players.get(*player) {
            if let Ok(grid) = node_grids.get(*node) {
                scroll.x = scroll
                    .x
                    .min(cursor.x * 3) // Keeps node cursor from going off the left
                    .max((cursor.x * 3 + 4).saturating_sub(size.width32())) // Keeps node cursor from going off the right
                    .min((grid.width() * 3 + 1).saturating_sub(size.width32())); // On resize, show as much grid as possible
                scroll.y = scroll
                    .y
                    .min(cursor.y * 2) // Keeps node cursor from going off the right
                    .min((grid.height() * 2 + 1).saturating_sub(size.height32())) // Keeps node cursor from going off the bottom
                    .max((cursor.y * 2 + 3).saturating_sub(size.height32())); // On resize, show as much grid as possible
            }
        }
    }
}
