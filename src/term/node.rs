mod render_node;

use crate::term::layout::StyleTty;
use crate::term::node::render_node::NodeViewScroll;
use crate::term::prelude::*;
use crate::term::{TerminalFocusMode, TerminalWindow};
use bevy::reflect::{FromReflect, Reflect};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
use game_core::{EntityGrid, Node};

use self::render_node::render_menu::calculate_action_menu_style;
use self::render_node::{GlyphRegistry, GridUi};

use super::layout::{CalculatedSizeTty, GlobalTranslationTty};
use super::render::RenderTtySet;

/// Event that tells us to show a specific Node entity
#[derive(Debug)]
pub struct ShowNode(pub Entity);

/// If there are multiple Nodes, this is the node that is being rendered to the screen
#[derive(Debug, Deref, DerefMut, Resource, Default)]
pub struct NodeFocus(pub Option<Entity>);

#[derive(Default)]
pub struct NodePlugin;

#[derive(Component, Debug, Deref)]
pub struct SelectedEntity(pub Option<Entity>);

impl Plugin for NodePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GlyphRegistry>()
            .init_resource::<NodeFocus>()
            .add_event::<ShowNode>()
            .add_system(create_node_ui.in_schedule(OnEnter(TerminalFocusMode::Node)))
            .add_system(node_cursor_controls.before(RenderTtySet::CalculateLayout))
            .add_systems(
                (calculate_action_menu_style,)
                    .in_set(OnUpdate(TerminalFocusMode::Node))
                    .in_set(RenderTtySet::PreCalculateLayout),
            )
            .add_systems(
                (
                    adjust_scroll.before(render_node::render_grid_system),
                    render_node::render_grid_system,
                    render_node::render_menu_system,
                    render_node::render_title_bar_system,
                    render_node::render_menu::action_menu,
                    render_node::render_menu::description,
                )
                    .in_set(OnUpdate(TerminalFocusMode::Node))
                    .in_set(RenderTtySet::PostCalculateLayout),
            );
    }
}

#[derive(Component, Debug, Default, Deref, DerefMut, FromReflect, Reflect)]
pub struct NodeCursor(pub UVec2);

pub fn adjust_scroll(
    mut node_cursors: Query<(&NodeCursor, &EntityGrid)>,
    mut grid_ui_view: Query<
        (
            &CalculatedSizeTty,
            &mut NodeViewScroll,
        ),
        With<GridUi>,
    >,
) {

    for (cursor, grid) in node_cursors.iter_mut() {
        if let Ok((size, mut scroll)) = grid_ui_view.get_single_mut() {
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

pub fn node_cursor_controls(
    mut node_cursors: Query<(&mut NodeCursor, &EntityGrid, &mut SelectedEntity)>,
    mut grid_ui_view: Query<
        (
            &CalculatedSizeTty,
            &GlobalTranslationTty,
            &NodeViewScroll,
        ),
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

pub fn create_node_ui(
    mut commands: Commands,
    mut show_node: EventReader<ShowNode>,
    mut terminal_window: ResMut<TerminalWindow>,
    mut node_focus: ResMut<NodeFocus>,
    nodes_without_cursors: Query<&EntityGrid, (With<Node>, Without<NodeCursor>)>,
) {
    use taffy::prelude::*;
    if let Some(ShowNode(node_id)) = show_node.iter().next() {
        if let Ok(grid) = nodes_without_cursors.get(*node_id) {
            commands.entity(*node_id).insert((
                NodeCursor::default(),
                SelectedEntity(grid.item_at(default())),
            ));
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
                        content_pane
                            .spawn((
                                StyleTty(taffy::prelude::Style {
                                    size: Size {
                                        width: Dimension::Points(13.),
                                        height: Dimension::Auto,
                                    },
                                    flex_direction: FlexDirection::Column,
                                    ..default()
                                }),
                                Name::new("Menu Bar"),
                            ))
                            .with_children(|menu_bar| {
                                menu_bar.spawn((
                                    StyleTty(taffy::prelude::Style {
                                        display: Display::None,
                                        size: Size {
                                            width: Dimension::Auto,
                                            height: Dimension::Auto,
                                        },
                                        flex_grow: 3.0,
                                        ..default()
                                    }),
                                    render_node::RenderMenu,
                                    Name::new("OldMenu"),
                                ));
                                menu_bar.spawn((
                                    StyleTty(taffy::prelude::Style {
                                        display: Display::None,
                                        size: Size {
                                            width: Dimension::Auto,
                                            height: Dimension::Points(0.0),
                                        },
                                        margin: Rect {
                                            bottom: Dimension::Points(1.0),
                                            ..Default::default()
                                        },
                                        ..default()
                                    }),
                                    render_node::render_menu::MenuUiActions,
                                    Name::new("Actions Menu"),
                                ));
                                menu_bar.spawn((
                                    StyleTty(taffy::prelude::Style {
                                        size: Size {
                                            width: Dimension::Auto,
                                            height: Dimension::Points(10.0),
                                        },
                                        flex_grow: 1.0,
                                        ..default()
                                    }),
                                    render_node::render_menu::MenuUiDescription,
                                    Name::new("DescriptionMenu"),
                                ));
                            });

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
                            render_node::GridUi,
                            NodeViewScroll::default(),
                        ));
                    });
                })
                .id();
            terminal_window.set_render_target(Some(render_root));
        }
        *node_focus = NodeFocus(Some(*node_id));
    }
}
