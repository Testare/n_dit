use crate::term::{
    layout::{CalculatedSizeTty, GlobalTranslationTty},
    prelude::*,
};
use game_core::EntityGrid;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};

use super::{grid_ui::GridUi, grid_ui::NodeViewScroll, NodeCursor, SelectedEntity};

pub fn node_cursor_controls(
    mut node_cursors: Query<(&mut NodeCursor, &EntityGrid, &mut SelectedEntity)>,
    mut grid_ui_view: Query<
        (&CalculatedSizeTty, &GlobalTranslationTty, &NodeViewScroll),
        With<GridUi>,
    >,
    mut inputs: EventReader<CrosstermEvent>,
) {
    for (mut cursor, grid, mut selected_entity) in node_cursors.iter_mut() {
        if let Ok((size, translation, scroll)) = grid_ui_view.get_single_mut() {
            for input in inputs.iter() {
                match input {
                    CrosstermEvent::Key(KeyEvent {
                        code: KeyCode::Char(input_char),
                        ..
                    }) => match input_char {
                        'k' | 'w' => {
                            cursor.y = cursor.y.saturating_sub(1);
                        }
                        'h' | 'a' => {
                            cursor.x = cursor.x.saturating_sub(1);
                        }
                        'j' | 's' => {
                            cursor.y = cursor.y.saturating_add(1).min(grid.height() - 1 as u32);
                        }
                        'l' | 'd' => {
                            cursor.x = cursor.x.saturating_add(1).min(grid.width() - 1 as u32);
                        }
                        _ => {}
                    },
                    CrosstermEvent::Mouse(
                        event @ MouseEvent {
                            kind,
                            column,
                            row,
                            modifiers,
                        },
                    ) => {
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
                                }
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
                                }
                                _ => {}
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
        let now_selected_entity = grid.item_at(**cursor);
        if selected_entity.0 != now_selected_entity {
            selected_entity.0 = now_selected_entity;
        }
    }
}
