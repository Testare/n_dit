mod render_node;

use crate::term::layout::StyleTty;
use crate::term::prelude::*;
use crate::term::{TerminalFocusMode, TerminalWindow};
use bevy::reflect::{FromReflect, Reflect};
use crossterm::event::{KeyCode, KeyEvent};
use game_core::{EntityGrid, Node};

use self::render_node::GlyphRegistry;

use super::render::RenderTtySet;

#[derive(Debug)]
pub struct ShowNode(pub Entity);

#[derive(Debug, Deref, DerefMut, Resource, Default)]
pub struct NodeFocus(pub Option<Entity>);

#[derive(Default)]
pub struct NodePlugin;

impl Plugin for NodePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GlyphRegistry>()
            .init_resource::<NodeFocus>()
            .add_event::<ShowNode>()
            .add_system(create_node_ui.in_schedule(OnEnter(TerminalFocusMode::Node)))
            .add_system(node_cursor_controls)
            .add_systems(
                (
                    render_node::render_grid_system,
                    render_node::render_menu_system,
                    render_node::render_title_bar_system,
                )
                    .in_set(OnUpdate(TerminalFocusMode::Node))
                    .in_set(RenderTtySet::RenderComponents),
            );
    }
}

#[derive(Component, Debug, Default, Deref, DerefMut, FromReflect, Reflect)]
pub struct NodeCursor(pub UVec2);

#[derive(Component, Debug, Default, Deref, DerefMut, FromReflect, Reflect)]
pub struct NodeViewScroll(pub UVec2);

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

pub fn create_node_ui(
    mut commands: Commands,
    mut show_node: EventReader<ShowNode>,
    mut terminal_window: ResMut<TerminalWindow>,
    mut node_focus: ResMut<NodeFocus>,
    nodes_without_cursors: Query<(), (With<Node>, Without<NodeCursor>)>,
) {
    use taffy::prelude::*;
    if let Some(ShowNode(node_id)) = show_node.iter().next() {
        if nodes_without_cursors.contains(*node_id) {
            commands.entity(*node_id).insert(NodeCursor::default());
        }
        if (*node_focus).is_none() {
            let render_root = commands
                .spawn((
                    StyleTty(taffy::prelude::Style {
                        size: Size {
                            width: Dimension::Points(100.),
                            height: Dimension::Points(100.),
                        },
                        flex_direction: FlexDirection::Column,
                        ..default()
                    }),
                    Name::new("Node UI Root"),
                    render_node::RenderNode,
                    crate::term::layout::LayoutRoot,
                ))
                .with_children(|root| {
                    root.spawn((
                        StyleTty(taffy::prelude::Style {
                            size: Size {
                                width: Dimension::Auto,
                                height: Dimension::Points(3.),
                            },
                            ..default()
                        }),
                        Name::new("Node Title Bar"),
                        render_node::RenderTitleBar,
                    ));
                    root.spawn((
                        StyleTty(taffy::prelude::Style {
                            size: Size {
                                width: Dimension::Auto,
                                height: Dimension::Auto,
                            },
                            flex_grow: 1.0,
                            ..default()
                        }),
                        Name::new("Node Content Pane"),
                    ))
                    .with_children(|content_pane| {
                        content_pane.spawn((
                            StyleTty(taffy::prelude::Style {
                                size: Size {
                                    width: Dimension::Points(13.),
                                    height: Dimension::Auto,
                                },
                                ..default()
                            }),
                            Name::new("Menu Bar"),
                            render_node::RenderMenu,
                        ));

                        content_pane.spawn((
                            StyleTty(taffy::prelude::Style {
                                size: Size {
                                    width: Dimension::Auto,
                                    height: Dimension::Auto,
                                },
                                border: Rect {
                                    left: Dimension::Points(1.0),
                                    ..default()
                                },
                                flex_grow: 1.0,

                                ..default()
                            }),
                            Name::new("Grid"),
                            render_node::RenderGrid,
                        ));
                    });
                })
                .id();
            terminal_window.set_render_target(Some(render_root));
        }
        *node_focus = NodeFocus(Some(*node_id));
    }
}
