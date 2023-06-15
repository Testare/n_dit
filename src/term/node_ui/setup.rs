use game_core::node::Node;
use game_core::player::ForPlayer;

use super::{NodeCursor, NodeFocus, ShowNode};
use crate::term::layout::{LayoutMouseTarget, StyleTty};
use crate::term::node_ui::grid_ui::{GridUi, NodeViewScroll};
use crate::term::node_ui::{AvailableMoves, SelectedAction, SelectedEntity};
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
    if let Some(ShowNode { player, node }) = show_node.iter().next() {
        if let Ok(grid) = nodes_without_cursors.get(*node) {
            commands.entity(*node).insert((
                NodeCursor::default(),
                SelectedEntity(grid.item_at(default())),
                SelectedAction(None),
                AvailableMoves::default(),
            ));
            commands.entity(*player).insert((
                NodeCursor::default(),
                SelectedEntity(grid.item_at(default())),
                // SelectedAction(None),
                // AvailableMoves::default(),
            ));
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
                        ForPlayer(*player),
                    ));
                    root.spawn((
                        StyleTty(taffy::prelude::Style {
                            size: Size {
                                width: Dimension::Auto,
                                height: Dimension::Auto,
                            },
                            flex_grow: 1.0,
                            flex_shrink: 0.0,
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
                                    ForPlayer(*player),
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
                                    super::menu_ui::MenuUiCardSelection::default(),
                                    ForPlayer(*player),
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
                                    ForPlayer(*player),
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
                                    ForPlayer(*player),
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
                                    ForPlayer(*player),
                                    Name::new("DescriptionMenu"),
                                ));
                            });

                        content_pane.spawn((
                            StyleTty(taffy::prelude::Style {
                                size: Size {
                                    width: Dimension::Auto,
                                    height: Dimension::Auto,
                                },
                                max_size: Size {
                                    width: Dimension::Points((grid.width() * 3 + 1) as f32),
                                    height: Dimension::Points((grid.height() * 2 + 1) as f32),
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
                            ForPlayer(*player),
                            NodeViewScroll::default(),
                        ));
                    });
                    root.spawn((
                        StyleTty(taffy::prelude::Style {
                            size: Size {
                                width: Dimension::Auto,
                                height: Dimension::Points(1.),
                            },
                            flex_shrink: 0.0,
                            ..default()
                        }),
                        Name::new("Message Bar"),
                        ForPlayer(*player),
                        super::MessageBarUi(vec!["Have you ever heard the story of Darth Plegius the wise? I thought not, it's not a story the jedi would tell you. He was powerful, some say he even could even stop people from dying. Of course, he was betrayed, and at this point Logan's memory starts to fail, and he isn't really able to quote the whole thing exactly. But of course I remember the gist.".to_owned()]),
                    ));
                })
                .id();

                terminal_window.set_render_target(Some(render_root));
            }
        }
        *node_focus = NodeFocus(Some(*node));
    }
}
