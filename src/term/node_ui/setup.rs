use game_core::{EntityGrid, Node};

use super::{NodeCursor, NodeFocus, ShowNode};
use crate::term::layout::{LayoutMouseTarget, StyleTty};
use crate::term::node_ui::grid_ui::{GridUi, NodeViewScroll};
use crate::term::node_ui::{AvailableMoves, SelectedEntity, SelectedAction};
use crate::term::prelude::*;
use crate::term::TerminalWindow;

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
                SelectedAction(None),
                AvailableMoves::default(),
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
                        super::titlebar_ui::TitleBarUi,
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
                                        width: Dimension::Points(14.),
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
                                    super::menu_ui::MenuUiLabel,
                                    Name::new("Menu Label"),
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
                                    LayoutMouseTarget,
                                    super::menu_ui::MenuUiCardSelection::<0>::default(),
                                    Name::new("Menu Card Selection"),
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
                                    super::menu_ui::MenuUiStats,
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
                                    super::menu_ui::MenuUiActions,
                                    LayoutMouseTarget,
                                    Name::new("Actions Menu"),
                                ));
                                menu_bar.spawn((
                                    StyleTty(taffy::prelude::Style {
                                        display: Display::None,
                                        min_size: Size {
                                            width: Dimension::Auto,
                                            height: Dimension::Points(0.0),
                                        },
                                        // flex_grow: 1.0,
                                        ..default()
                                    }),
                                    super::menu_ui::MenuUiDescription,
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
                            GridUi,
                            LayoutMouseTarget,
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
