mod render_node;

use crate::term::prelude::*;
use crate::term::{TerminalFocusMode, TerminalWindow};
use bevy::reflect::{FromReflect, Reflect};
use crossterm::event::{KeyCode, KeyEvent};
use game_core::{EntityGrid, Node};

use self::render_node::GlyphRegistry;

#[derive(Default)]
pub struct NodePlugin;

impl Plugin for NodePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GlyphRegistry>()
            // In the future, these can be added to a state only for node
            .add_system(node_on_focus.in_schedule(OnEnter(TerminalFocusMode::Node)))
            .add_systems(
                (node_cursor_controls, render_node::render_node)
                    .in_set(OnUpdate(TerminalFocusMode::Node)),
            );
    }
}

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
                commands
                    .entity(*render_target)
                    .insert(NodeCursor::default());
            }
        }
    }
}

pub fn node_cursor_controls(
    mut node_cursors: Query<(&mut NodeCursor, &EntityGrid)>,
    mut inputs: EventReader<CrosstermEvent>,
) {
    for (mut cursor, grid) in node_cursors.iter_mut() {
        for input in inputs.iter() {
            if let CrosstermEvent::Key(KeyEvent {
                code: KeyCode::Char(input_char),
                ..
            }) = input
            {
                match input_char {
                    'k' | 'w' => cursor.y = cursor.y.saturating_sub(1),
                    'h' | 'a' => cursor.x = cursor.x.saturating_sub(1),
                    'j' | 's' => {
                        cursor.y = cursor.y.saturating_add(1).min(grid.height() - 1 as u32)
                    }
                    'l' | 'd' => cursor.x = cursor.x.saturating_add(1).min(grid.width() - 1 as u32),
                    _ => {}
                }
            }
        }
    }
}
