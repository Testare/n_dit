use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
use game_core::player::{Player, ForPlayer};

use super::grid_ui::{GridUi, NodeViewScroll};
use super::{MessageBarUi, NodeCursor, SelectedAction, SelectedEntity};
use crate::term::layout::{CalculatedSizeTty, GlobalTranslationTty};
use crate::term::prelude::*;

pub fn node_cursor_controls(
    mut node_cursors: Query<(
        &mut NodeCursor,
        &EntityGrid,
        &mut SelectedAction,
        &mut SelectedEntity,
    ), Without<Player>>,
    mut player_q: Query<(&mut SelectedEntity,), With<Player>>,
    mut message_bar_ui: Query<&mut MessageBarUi>,
    mut grid_ui_view: Query<
        (&CalculatedSizeTty, &GlobalTranslationTty, &NodeViewScroll, &ForPlayer),
        With<GridUi>,
    >,
    mut inputs: EventReader<CrosstermEvent>,
) {
    for (mut cursor, grid, mut selected_action, mut selected_entity) in node_cursors.iter_mut() {
        if let Ok((size, translation, scroll, ForPlayer(player))) = grid_ui_view.get_single_mut() {
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
            if let Ok((mut selected_entity,)) = player_q.get_mut(*player) {
                let now_selected_entity = grid.item_at(**cursor);
                if selected_entity.0 != now_selected_entity {
                    selected_entity.0 = now_selected_entity;
                    **selected_action = None;
                }

            }
        }
        // TO BE REPLACED
        let now_selected_entity = grid.item_at(**cursor);
        if selected_entity.0 != now_selected_entity {
            selected_entity.0 = now_selected_entity;
            **selected_action = None;
        }
    }
}
