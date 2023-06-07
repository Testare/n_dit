use game_core::{prelude::*, EntityGrid, Node};

use crate::term::{
    layout::StyleTty,
    node_ui::{render_node::NodeViewScroll, SelectedEntity},
    TerminalWindow,
};

use super::{NodeCursor, NodeFocus, ShowNode};

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
                    super::render_node::NodeUiRoot,
                    crate::term::layout::LayoutRoot,
                ))
                .with_children(|root| {
                    root.spawn((
                        StyleTty(taffy::prelude::Style {
                            size: Size {
                                width: Dimension::Auto,
                                height: Dimension::Points(3.),
                            },
                            flex_shrink: 0.0,
                            ..default()
                        }),
                        Name::new("Node Title Bar"),
                        super::render_node::TitleBarUi,
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
                                        min_size: Size {
                                            width: Dimension::Auto,
                                            height: Dimension::Points(0.0),
                                        },
                                        ..default()
                                    }),
                                    super::render_node::render_menu::MenuUiLabel,
                                    Name::new("Menu Label"),
                                ));
                                menu_bar.spawn((
                                    StyleTty(taffy::prelude::Style {
                                        display: Display::None,
                                        min_size: Size {
                                            width: Dimension::Auto,
                                            height: Dimension::Points(0.0),
                                        },
                                        ..default()
                                    }),
                                    super::render_node::render_menu::MenuUiStats,
                                    Name::new("Menu Stats"),
                                ));
                                menu_bar.spawn((
                                    StyleTty(taffy::prelude::Style {
                                        display: Display::None,
                                        min_size: Size {
                                            width: Dimension::Auto,
                                            height: Dimension::Points(0.0),
                                        },
                                        ..default()
                                    }),
                                    super::render_node::render_menu::MenuUiActions,
                                    Name::new("Actions Menu"),
                                ));
                                menu_bar.spawn((
                                    StyleTty(taffy::prelude::Style {
                                        display: Display::None,
                                        min_size: Size {
                                            width: Dimension::Auto,
                                            height: Dimension::Points(0.0),
                                        },
                                        flex_grow: 1.0,
                                        ..default()
                                    }),
                                    super::render_node::render_menu::MenuUiDescription,
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
                            super::render_node::GridUi,
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
