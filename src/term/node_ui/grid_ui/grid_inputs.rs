use std::ops::Deref;

use crossterm::event::{MouseButton, MouseEventKind};
use game_core::node::NodeOp;
use game_core::player::{ForPlayer, Player};

use super::{GridUi, Scroll2D};
use crate::term::layout::LayoutEvent;
use crate::term::node_ui::NodeCursor;
use crate::term::prelude::*;

pub fn handle_layout_events(
    mut ev_mouse: EventReader<LayoutEvent>,
    ui: Query<(&ForPlayer, &Scroll2D), With<GridUi>>,
    mut players: Query<&mut NodeCursor, With<Player>>,
    mut node_command: EventWriter<Op<NodeOp>>,
) {
    for event in ev_mouse.iter() {
        if let Ok((ForPlayer(player), scroll)) = ui.get(event.entity()) {
            if let MouseEventKind::Down(button) = event.event_kind() {
                log::debug!("Clicked on the grid");
                let mut cursor = players
                    .get_mut(*player)
                    .expect("a player should have a node cursor if there is a grid ui");
                let clicked_pos = event.pos() + **scroll;
                let clicked_node_pos = UVec2 {
                    x: clicked_pos.x / 3,
                    y: clicked_pos.y / 2,
                };

                if **cursor.deref() != clicked_node_pos {
                    **cursor = clicked_node_pos;
                }

                match button {
                    MouseButton::Right => {
                        log::debug!("That was a right click");
                    },
                    _ => {},
                }
            }
        }
    }
}
