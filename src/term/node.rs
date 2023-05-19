use crate::term::prelude::*;
use game_core::{Node, EntityGrid};
use bevy::reflect::{FromReflect, Reflect};
use crate::term::TerminalWindow;
use crossterm::event::{KeyEvent, KeyCode};

#[derive(Component, Debug, Default, Deref, DerefMut, FromReflect, Reflect)]
pub struct NodeCursor(pub UVec2);

pub fn node_on_focus(
    mut commands: Commands,
    terminal_windows: Query<&TerminalWindow, (Without<NodeCursor>, Changed<TerminalWindow>)>,
    nodes: Query<Entity, With<Node>>,
) {
    for tw in terminal_windows.iter() {
        if let Some(render_target) = tw.render_target() {
            if nodes.contains(*render_target) {
                commands.entity(*render_target).insert(
                    NodeCursor::default()
                );
            }
        }
    }
}


pub fn node_cursor_controls(
    mut node_cursors: Query<(&mut NodeCursor, &EntityGrid)>,
    mut inputs: EventReader<CrosstermEvent>,
) {
    for (mut cursor, grid) in node_cursors.iter_mut() {
        for input in inputs.iter()  {
            if let CrosstermEvent::Key(KeyEvent {
                code: KeyCode::Char(input_char),
                ..
            }) = input {
                match input_char {
                    'k' | 'w' => cursor.y = cursor.y.saturating_sub(1),
                    'h' | 'a' => cursor.x = cursor.x.saturating_sub(1),
                    'j' | 's' => cursor.y = cursor.y.saturating_add(1).min(grid.height() - 1 as u32),
                    'l' | 'd' => cursor.x = cursor.x.saturating_add(1).min(grid.width() - 1 as u32),
                    _ => {},
                }
            }
        }
    }
}
