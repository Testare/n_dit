use bevy::app::AppExit;
use crossterm::style::{ContentStyle, Stylize};
use game_core::node::{InNode, Node, NodeBattleIntelligence, NodeOp};
use game_core::op::CoreOps;
use game_core::player::{ForPlayer, Player};
use getset::CopyGetters;
use unicode_width::UnicodeWidthStr;

use super::{NodeCursor, NodeUiQ};
use crate::animation::AnimationPlayer;
use crate::base_ui::context_menu::{ContextAction, ContextActions};
use crate::base_ui::{ButtonUiBundle, FlexibleTextUi, PopupMenu, Tooltip, TooltipBar};
use crate::input_event::{MouseEventListener, MouseEventTtyDisabled};
use crate::layout::{StyleTty, UiFocusBundle, UiFocusCycleOrder, VisibilityTty};
use crate::node_ui::button_ui::{
    EndTurnButton, HelpButton, OptionsButton, QuitButton, ReadyButton,
};
use crate::node_ui::grid_ui::{GridUi, GridUiAnimation};
use crate::node_ui::menu_ui::{
    MenuUiActions, MenuUiCardSelection, MenuUiDescription, MenuUiLabel, MenuUiStats,
};
use crate::node_ui::node_popups::{help_msg, HelpMenu, OptionsMenu};
use crate::node_ui::{
    AvailableActionTargets, AvailableMoves, CursorIsHidden, HasNodeUi, NodeUi, SelectedAction,
    SelectedEntity, TelegraphedAction,
};
use crate::prelude::*;
use crate::render::TerminalRendering;
use crate::{KeyMap, Submap, TerminalWindow};

#[derive(Debug, Resource, CopyGetters)]
pub struct ButtonContextActions {
    #[getset(get_copy = "pub")]
    start: Entity,
    #[getset(get_copy = "pub")]
    quit: Entity,
}

impl FromWorld for ButtonContextActions {
    fn from_world(world: &mut World) -> Self {
        let start = world
            .spawn(ContextAction::new(
                "Start battle".to_string(),
                |id, world| {
                    // TODO make a factory method for this closure
                    let for_player = world.get::<ForPlayer>(id).copied();
                    if let Some(ForPlayer(player_id)) = for_player {
                        world
                            .get_resource_mut::<CoreOps>()
                            .expect("should have CoreOps initialized")
                            .request(player_id, NodeOp::ReadyToGo);
                    }
                },
            ))
            .id();
        let quit = world
            .spawn(ContextAction::new("Quit game".to_string(), |_, world| {
                world.send_event(AppExit);
            }))
            .id();
        ButtonContextActions { start, quit }
    }
}

pub fn create_node_ui(
    mut commands: Commands,
    res_button_context_actions: Res<ButtonContextActions>,
    player_now_in_node: Query<
        (Entity, AsDeref<InNode>),
        (With<Player>, Added<InNode>, Without<NodeBattleIntelligence>),
    >,
    mut terminal_window: ResMut<TerminalWindow>,
    mut players: Query<&mut KeyMap>,
    node_qs: Query<(NodeUiQ, &Name), With<Node>>,
) {
    use taffy::prelude::*;
    for (player, node) in player_now_in_node.iter() {
        if let Ok((node_q, node_name)) = node_qs.get(*node) {
            if let Ok(mut key_map) = players.get_mut(player) {
                key_map.activate_submap(Submap::Node);
            }

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
                    Name::new(format!("Node UI Root - {player:?}")),
                    crate::layout::LayoutRoot,
                    TerminalRendering::default(),
                ))
                .with_children(|root| {
                    root.spawn(super::titlebar_ui::TitleBarUi::bundle(player, &node_q))
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
                                        justify_content: Some(JustifyContent::Center),
                                        ..default()
                                    }),
                                    Name::new("Title Bar Right"),
                                ))
                                .with_children(|title_bar_right| {
                                    title_bar_right.spawn((
                                        ButtonUiBundle::new("Options", ContentStyle::new().green()),
                                        ForPlayer(player),
                                        OptionsButton,
                                        Tooltip::new("[Esc] Opens menu for options"),
                                    ));

                                    title_bar_right.spawn((
                                        StyleTty(taffy::prelude::Style {
                                            size: Size {
                                                width: Dimension::Points(1.0),
                                                height: Dimension::Auto,
                                            },
                                            min_size: Size { ..TaffyZero::ZERO },
                                            flex_grow: 0.0,
                                            flex_shrink: 2.0,
                                            ..default()
                                        }),
                                    ));


                                    title_bar_right.spawn((
                                        ForPlayer(player),
                                        ReadyButton,
                                        ButtonUiBundle::new("Ready", ContentStyle::new().blue()),
                                        MouseEventTtyDisabled,
                                        ContextActions::new(player, vec![res_button_context_actions.start()]),
                                        VisibilityTty(true),
                                        Tooltip::new("[-] When you've placed all your units, click here to begin")
                                    ));
                                    title_bar_right.spawn((
                                        ForPlayer(player),
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
                                        ForPlayer(player),
                                        HelpButton,
                                        Tooltip::new("[?] Open guide to the game"),
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
                                        ContextActions::new(player, vec![res_button_context_actions.quit()]),
                                        QuitButton,
                                        Tooltip::new("[q] Click to exit")
                                    ));
                                });
                        });
                    root.spawn((
                        ForPlayer(player),
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
                        MouseEventListener,
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
                                menu_bar.spawn(MenuUiLabel::bundle(player, &node_q));
                                menu_bar
                                    .spawn((
                                        MenuUiCardSelection::bundle(player, &node_q),
                                        UiFocusCycleOrder(2),
                                    ));
                                menu_bar.spawn(MenuUiStats::bundle(player, &node_q));
                                menu_bar
                                    .spawn((
                                        MenuUiActions::bundle(player, &node_q),
                                        UiFocusCycleOrder(1),
                                    ));
                                menu_bar.spawn(MenuUiDescription::bundle(player, &node_q));
                            });
                        content_pane
                            .spawn((
                                GridUi::bundle(player, &node_q),
                                UiFocusCycleOrder(0),
                            ))
                            .with_children(|grid_ui| {
                                grid_ui.spawn(StyleTty::buffer());
                                grid_ui.spawn(
                                    StyleTty(taffy::prelude::Style {
                                        flex_direction: FlexDirection::Column,
                                        ..default()
                                    })
                                ).with_children(|grid_ui_center| {
                                    grid_ui_center.spawn(StyleTty::buffer());
                                    grid_ui_center.spawn((
                                        TerminalRendering::default(),
                                        Name::new("Node popup menu"),
                                        StyleTty(taffy::prelude::Style {
                                            flex_grow: 0.0,
                                            padding: Rect::points(1.0),
                                            ..default()
                                        }),
                                        PopupMenu,
                                    )).with_children(|popup_menu| {
                                        popup_menu.spawn((
                                            TerminalRendering::new(vec!["I love you,".to_string(), "my dear wife!".to_string()]),
                                            StyleTty(taffy::prelude::Style {
                                                size: Size {
                                                    width: Dimension::Points(13.0),
                                                    height: Dimension::Points(2.0),
                                                },
                                                ..default()
                                            }),
                                            VisibilityTty(false),
                                        ));
                                        let help_msg = help_msg();
                                        popup_menu.spawn((
                                            ForPlayer(player),
                                            HelpMenu,
                                            Name::new("Help menu"),
                                            StyleTty(taffy::prelude::Style {
                                                size: Size {
                                                    width: Dimension::Points(help_msg.width() as f32),
                                                    height: Dimension::Points(help_msg.height() as f32),
                                                },
                                                ..default()
                                            }),
                                            TerminalRendering::from(help_msg.clone()),
                                            VisibilityTty(false),
                                        ));
                                        popup_menu.spawn((
                                            ForPlayer(player),
                                            Name::new("Options menu"),
                                            OptionsMenu,
                                            StyleTty(taffy::prelude::Style {
                                                size: Size {
                                                    width: Dimension::Points(13.0),
                                                    height: Dimension::Points(2.0),
                                                },
                                                ..default()
                                            }),
                                            TerminalRendering::new(vec!["Options!".to_string()]),
                                            VisibilityTty(false),
                                        ));
                                    });
                                    grid_ui_center.spawn(StyleTty::buffer());
                                });
                                grid_ui.spawn(StyleTty::buffer());
                            });
                    });
                    root.spawn(super::MessageBarUi::bundle(player, &node_q));
                })
                .id();
            commands.spawn((
                Name::new("GridAnimationPlayer"),
                GridUiAnimation,
                ForPlayer(player),
                AnimationPlayer::default(),
                TerminalRendering::default(),
            ));

            commands
                .entity(player)
                .insert((
                    NodeCursor::default(),
                    CursorIsHidden::default(),
                    SelectedEntity(node_q.grid.item_at(default())),
                    SelectedAction(None),
                    TelegraphedAction(None),
                    AvailableActionTargets::default(),
                    UiFocusBundle::default(),
                    AvailableMoves::default(),
                    HasNodeUi,
                ))
                .log_components();

            terminal_window.set_render_target(Some(render_root));
        }
    }
}
