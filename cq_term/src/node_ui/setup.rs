use crossterm::style::{ContentStyle, Stylize};
use game_core::node::Node;
use game_core::player::ForPlayer;
use unicode_width::UnicodeWidthStr;

use super::{NodeCursor, NodeUiQ, ShowNode};
use crate::base_ui::{ButtonUiBundle, FlexibleTextUi, Tooltip, TooltipBar};
use crate::layout::{
    CalculatedSizeTty, LayoutMouseTarget, LayoutMouseTargetDisabled, StyleTty, UiFocusBundle,
    UiFocusCycleOrder, VisibilityTty,
};
use crate::node_ui::button_ui::{EndTurnButton, HelpButton, PauseButton, QuitButton, ReadyButton};
use crate::node_ui::grid_ui::GridUi;
use crate::node_ui::menu_ui::{
    MenuUiActions, MenuUiCardSelection, MenuUiDescription, MenuUiLabel, MenuUiStats,
};
use crate::node_ui::{
    AvailableActionTargets, AvailableMoves, HasNodeUi, NodeUi, SelectedAction, SelectedEntity,
};
use crate::prelude::*;
use crate::render::TerminalRendering;
use crate::TerminalWindow;

pub fn create_node_ui(
    mut commands: Commands,
    mut show_node: EventReader<ShowNode>,
    mut terminal_window: ResMut<TerminalWindow>,
    node_qs: Query<(NodeUiQ, &Name), With<Node>>,
) {
    use taffy::prelude::*;
    if let Some(ShowNode { player, node }) = show_node.iter().next() {
        if let Ok((node_q, node_name)) = node_qs.get(*node) {
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
                    crate::layout::LayoutRoot,
                    TerminalRendering::default(),
                ))
                .with_children(|root| {
                    root.spawn(super::titlebar_ui::TitleBarUi::bundle(*player, &node_q))
                        .with_children(|title_bar| {
                            title_bar.spawn((
                                StyleTty(taffy::prelude::Style {
                                    size: Size {
                                        width: Dimension::Auto,
                                        height: Dimension::Auto,
                                    },
                                    flex_grow: 1.0,
                                    ..default()
                                }),
                                Name::new("Title Bar Left"),
                            ));
                            title_bar.spawn((
                                StyleTty(taffy::prelude::Style {
                                    size: Size {
                                        width: Dimension::Points(node_name.as_str().width() as f32),
                                        height: Dimension::Auto,
                                    },
                                    flex_grow: 0.0,
                                    flex_shrink: 1.0,
                                    ..default()
                                }),
                                TerminalRendering::new(vec![node_name.to_string()]),
                                Name::new("Node Title"),
                            ));
                            title_bar
                                .spawn((
                                    StyleTty(taffy::prelude::Style {
                                        size: Size {
                                            width: Dimension::Auto,
                                            height: Dimension::Auto,
                                        },
                                        max_size: Size {
                                            width: Dimension::Points(60.0),
                                            height: Dimension::Auto,
                                        },
                                        flex_grow: 1.0,
                                        justify_content: JustifyContent::Center,
                                        ..default()
                                    }),
                                    Name::new("Title Bar Right"),
                                ))
                                .with_children(|title_bar_right| {
                                    title_bar_right.spawn((
                                        ButtonUiBundle::new("Pause", ContentStyle::new().green()),
                                        PauseButton,
                                        Tooltip::new("[Escape] TODO Puase the agme and open pause menu")
                                    ));

                                    title_bar_right.spawn((StyleTty(taffy::prelude::Style {
                                        size: Size {
                                            width: Dimension::Points(1.0),
                                            height: Dimension::Auto,
                                        },
                                        min_size: Size { ..default() },
                                        flex_grow: 0.0,
                                        flex_shrink: 2.0,
                                        ..default()
                                    }),));

                                    title_bar_right.spawn((
                                        ForPlayer(*player),
                                        ReadyButton,
                                        ButtonUiBundle::new("Ready", ContentStyle::new().blue()),
                                        LayoutMouseTargetDisabled,
                                        VisibilityTty(true),
                                        Tooltip::new("[-] When you've placed all your units, click here to begin")
                                    ));

                                    title_bar_right.spawn((
                                        ForPlayer(*player),
                                        EndTurnButton,
                                        ButtonUiBundle::new("End Turn", ContentStyle::new().blue()),
                                        VisibilityTty::invisible(),
                                        Tooltip::new("[-] End your turn and let the next player go")
                                    ));

                                    title_bar_right.spawn((StyleTty(taffy::prelude::Style {
                                        size: Size {
                                            width: Dimension::Points(1.0),
                                            height: Dimension::Auto,
                                        },
                                        flex_grow: 0.0,
                                        flex_shrink: 2.0,
                                        ..default()
                                    }),));

                                    title_bar_right.spawn((
                                        ButtonUiBundle::new("Help", ContentStyle::new().yellow()),
                                        HelpButton,
                                        Tooltip::new("[?] TODO Open guide to the game")
                                    ));

                                    title_bar_right.spawn((StyleTty(taffy::prelude::Style {
                                        size: Size {
                                            width: Dimension::Points(1.0),
                                            height: Dimension::Auto,
                                        },
                                        flex_grow: 0.0,
                                        flex_shrink: 2.0,
                                        ..default()
                                    }),));

                                    title_bar_right.spawn((
                                        ButtonUiBundle::new("Quit", ContentStyle::new().red()),
                                        QuitButton,
                                        Tooltip::new("[q] Click to exit")
                                    ));
                                });
                        });
                    root.spawn((
                        ForPlayer(*player),
                        Name::new("Tooltip bar"),
                        TooltipBar,
                        FlexibleTextUi {
                            style: ContentStyle::new().cyan(),
                            text: "<Tooltip bar>".to_string(),
                        },
                        TerminalRendering::default(),
                        StyleTty(taffy::prelude::Style {
                            size: Size {
                                width: Dimension::Auto,
                                height: Dimension::Points(2.0),
                            },
                            flex_grow: 0.0,
                            ..default()
                        }),
                        LayoutMouseTarget,
                        VisibilityTty(true),
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
                                menu_bar.spawn(MenuUiLabel::bundle(*player, &node_q));
                                menu_bar
                                    .spawn(MenuUiCardSelection::bundle(*player, &node_q))
                                    .insert(UiFocusCycleOrder(2));
                                menu_bar.spawn(MenuUiStats::bundle(*player, &node_q));
                                menu_bar
                                    .spawn(MenuUiActions::bundle(*player, &node_q))
                                    .insert(UiFocusCycleOrder(1));
                                menu_bar.spawn(MenuUiDescription::bundle(*player, &node_q));
                            });
                        content_pane
                            .spawn(GridUi::bundle(*player, &node_q))
                            .insert(UiFocusCycleOrder(0));
                    });
                    root.spawn(super::MessageBarUi::bundle(*player, &node_q));
                })
                .id();

            commands.entity(*player).insert((
                NodeCursor::default(),
                SelectedEntity(node_q.grid.item_at(default())),
                SelectedAction(None),
                AvailableActionTargets::default(),
                UiFocusBundle::default(),
                AvailableMoves::default(),
                HasNodeUi,
            ));
            terminal_window.set_render_target(Some(render_root));
        }
    }
}
