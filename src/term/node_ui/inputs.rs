use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
use game_core::node::{InNode, Node};
use game_core::player::{ForPlayer, Player};

use super::grid_ui::{GridUi, NodeViewScroll};
use super::{MessageBarUi, NodeCursor, SelectedAction, SelectedEntity};
use crate::term::layout::{CalculatedSizeTty, GlobalTranslationTty};
use crate::term::prelude::*;

pub fn node_cursor_controls(
    node_grids: Query<&EntityGrid, With<Node>>,
    mut players: Query<
        (
            &InNode,
            &mut NodeCursor,
            &mut SelectedEntity,
            &mut SelectedAction,
        ),
        With<Player>,
    >,
    mut message_bar_ui: Query<&mut MessageBarUi>,
    grid_ui_view: Query<
        (
            &CalculatedSizeTty,
            &GlobalTranslationTty,
            &NodeViewScroll,
            &ForPlayer,
        ),
        With<GridUi>,
    >,
    mut inputs: EventReader<CrosstermEvent>,
) {
    for (size, translation, scroll, ForPlayer(player)) in grid_ui_view.iter() {
        if let Ok((InNode(node), mut cursor, mut selected_entity, mut selected_action)) =
            players.get_mut(*player)
        {
            let grid = node_grids
                .get(*node)
                .expect("if a player is in a node, it should have an entity grid");

            for input in inputs.iter() {
                match input {
                    CrosstermEvent::Key(KeyEvent {
                        code: KeyCode::Char(input_char),
                        ..
                    }) => match input_char {
                        'k' | 'w' => {
                            cursor.y = cursor.y.saturating_sub(1);
                        },
                        'h' | 'a' => {
                            cursor.x = cursor.x.saturating_sub(1);
                        },
                        'j' | 's' => {
                            cursor.y = cursor.y.saturating_add(1).min(grid.height() - 1 as u32);
                        },
                        'l' | 'd' => {
                            cursor.x = cursor.x.saturating_add(1).min(grid.width() - 1 as u32);
                        },
                        _ => {},
                    },

                    CrosstermEvent::Key(KeyEvent {
                        code: KeyCode::Enter,
                        ..
                    }) => {
                        // Next message
                        for mut msg_bar in message_bar_ui.iter_mut() {
                            msg_bar.0 = msg_bar.0[1..].into();
                        }
                    },
                    CrosstermEvent::Mouse(
                        event @ MouseEvent {
                            kind,
                            column,
                            row,
                            modifiers,
                        },
                    ) => {
                        // TODO Use layout events instead
                        let grid_contained = size.contains_mouse_event(translation, event);
                        if grid_contained {
                            match *kind {
                                MouseEventKind::Moved
                                    if modifiers.contains(KeyModifiers::SHIFT) =>
                                {
                                    let new_x = ((*column as u32) + scroll.x - translation.x) / 3;
                                    let new_y = ((*row as u32) + scroll.y - translation.y) / 2;
                                    if new_x < grid.width() && new_y < grid.height() {
                                        cursor.x = new_x;
                                        cursor.y = new_y;
                                    }
                                },
                                MouseEventKind::Down(MouseButton::Left) => {
                                    let new_x = ((*column as u32) + scroll.x - translation.x) / 3;
                                    let new_y = ((*row as u32) + scroll.y - translation.y) / 2;
                                    if new_x < grid.width() && new_y < grid.height() {
                                        if cursor.x == new_x && cursor.y == new_y {
                                            log::debug!("Click again on selected square")
                                        } else {
                                            cursor.x = new_x;
                                            cursor.y = new_y;
                                        }
                                    }
                                },
                                _ => {},
                            }
                        }
                    },
                    _ => {},
                }
            }
            let now_selected_entity = grid.item_at(**cursor);
            if selected_entity.0 != now_selected_entity {
                selected_entity.0 = now_selected_entity;
                **selected_action = None;
            }
        }
    }
}
