use bevy::app::AppExit;
use bevy::ecs::system::RunSystemOnce;
use crossterm::style::{ContentStyle, Stylize};
use game_core::card::NO_OP_ACTION_ID;
use game_core::node::{InNode, Node, NodeBattleIntelligence, NodeOp};
use game_core::op::CoreOps;
use game_core::player::{ForPlayer, Player};
use getset::CopyGetters;
use unicode_width::UnicodeWidthStr;

use super::{NodeCursor, NodeUiQ};
use crate::animation::AnimationPlayer;
use crate::base_ui::context_menu::{ContextAction, ContextActions};
use crate::base_ui::{ButtonUiBundle, FlexibleTextUi, PopupMenu, Tooltip, TooltipBar};
use crate::dialogue_ui::DialogueMenu;
use crate::input_event::{MouseEventListener, MouseEventTtyDisabled};
use crate::layout::{StyleTty, UiFocusBundle, UiFocusCycleOrder, VisibilityTty};
use crate::linkage::base_ui_game_core;
use crate::main_ui::{MainUiOp, UiOps};
use crate::node_ui::button_ui::{
    EndTurnButton, HelpButton, OptionsButton, QuitButton, ReadyButton, UndoButton,
};
use crate::node_ui::grid_ui::{GridUi, GridUiAnimation};
use crate::node_ui::menu_ui::{
    MenuUiActions, MenuUiCardSelection, MenuUiDescription, MenuUiLabel, MenuUiStats,
};
use crate::node_ui::node_popups::{help_msg, HelpMenu, OptionsMenu};
use crate::node_ui::{
    AvailableActionTargets, AvailableMoves, CursorIsHidden, HasNodeUi, NodeUi, NodeUiScreen,
    SelectedAction, SelectedNodePiece, TelegraphedAction,
};
use crate::prelude::*;
use crate::render::TerminalRendering;
use crate::{KeyMap, Submap};

#[derive(Debug, Resource, CopyGetters)]
pub struct ButtonContextActions {
    #[getset(get_copy = "pub")]
    end_turn: Entity,
    #[getset(get_copy = "pub")]
    quit: Entity,
    #[getset(get_copy = "pub")]
    start: Entity,
    #[getset(get_copy = "pub")]
    toggle_help: Entity,
    #[getset(get_copy = "pub")]
    toggle_options: Entity,
    #[getset(get_copy = "pub")]
    undo: Entity,
}

impl FromWorld for ButtonContextActions {
    fn from_world(world: &mut World) -> Self {
        let start = world
            .spawn(base_ui_game_core::context_action_from_op::<CoreOps, _>(
                "Start battle",
                NodeOp::ReadyToGo,
            ))
            .id();
        let end_turn = world
            .spawn(base_ui_game_core::context_action_from_op::<CoreOps, _>(
                "End Player Phase",
                NodeOp::EndTurn,
            ))
            .id();
        let _no_op = world
            .spawn(base_ui_game_core::context_action_from_op::<CoreOps, _>(
                "End Turn",
                NodeOp::PerformCurioAction {
                    action_id: NO_OP_ACTION_ID,
                    curio: None,
                    target: default(),
                },
            ))
            .id();
        let toggle_options = world
            .spawn((ContextAction::new(
                "Toggle Options menu",
                |id, world: &mut World| {
                    if let Some(&ForPlayer(player_id)) = world.get::<ForPlayer>(id) {
                        world.run_system_once(
                            move |mut help_menu_q: Query<
                                (AsDerefCopied<ForPlayer>, AsDerefMut<VisibilityTty>),
                                (With<HelpMenu>, Without<OptionsMenu>),
                            >,
                                  mut options_menu_q: Query<
                                (AsDerefCopied<ForPlayer>, AsDerefMut<VisibilityTty>),
                                (With<OptionsMenu>, Without<HelpMenu>),
                            >| {
                                if let Some((_, mut help_vis)) = help_menu_q
                                    .iter_mut()
                                    .find(|(for_player, _)| *for_player == player_id)
                                {
                                    help_vis.set_if_neq(false);
                                }
                                if let Some((_, mut options_vis)) = options_menu_q
                                    .iter_mut()
                                    .find(|(for_player, _)| *for_player == player_id)
                                {
                                    *options_vis = !*options_vis;
                                }
                            },
                        );
                    }
                },
            ),))
            .id();
        let toggle_help = world
            .spawn((ContextAction::new(
                "Toggle Help",
                |id, world: &mut World| {
                    if let Some(&ForPlayer(player_id)) = world.get::<ForPlayer>(id) {
                        world.run_system_once(
                            move |mut help_menu_q: Query<
                                (AsDerefCopied<ForPlayer>, AsDerefMut<VisibilityTty>),
                                (With<HelpMenu>, Without<OptionsMenu>),
                            >,
                                  mut options_menu_q: Query<
                                (AsDerefCopied<ForPlayer>, AsDerefMut<VisibilityTty>),
                                (With<OptionsMenu>, Without<HelpMenu>),
                            >| {
                                if let Some((_, mut help_vis)) = help_menu_q
                                    .iter_mut()
                                    .find(|(for_player, _)| *for_player == player_id)
                                {
                                    *help_vis = !*help_vis;
                                }
                                if let Some((_, mut options_vis)) = options_menu_q
                                    .iter_mut()
                                    .find(|(for_player, _)| *for_player == player_id)
                                {
                                    options_vis.set_if_neq(false);
                                }
                            },
                        );
                    }
                },
            ),))
            .id();
        let quit = world
            .spawn(ContextAction::new("Quit game".to_string(), |_, world| {
                world.send_event(AppExit);
            }))
            .id();
        let undo = world
            .spawn(base_ui_game_core::context_action_from_op::<CoreOps, _>(
                "Undo",
                NodeOp::Undo,
            ))
            .id();
        ButtonContextActions {
            end_turn,
            quit,
            start,
            toggle_help,
            toggle_options,
            undo,
        }
    }
}

pub fn create_node_ui(
    mut commands: Commands,
    res_button_context_actions: Res<ButtonContextActions>,
    mut res_ui_ops: ResMut<UiOps>,
    player_now_in_node: Query<
        (Entity, AsDeref<InNode>),
        (With<Player>, Added<InNode>, Without<NodeBattleIntelligence>),
    >,
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
                        grid_row: line(1),
                        grid_column: line(1),
                        flex_direction: FlexDirection::Column,
                        ..default()
                    }),
                    Name::new(format!("Node UI Root - {player:?}")),
                    NodeUiScreen,
                    ForPlayer(player),
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
                                        ForPlayer(player),
                                        UndoButton,
                                        ButtonUiBundle::new("Undo", ContentStyle::new().cyan()),
                                        ContextActions::new(player, vec![res_button_context_actions.undo()]),
                                        VisibilityTty::invisible(),
                                        Tooltip::new("[⌫] Undo")
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
                                        ContextActions::new(player, vec![res_button_context_actions.end_turn()]),
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
                                        ButtonUiBundle::new("Options", ContentStyle::new().green()),
                                        ForPlayer(player),
                                        OptionsButton,
                                        ContextActions::new(player, vec![res_button_context_actions.toggle_options()]),
                                        Tooltip::new("[Esc] Opens menu for options"),
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
                                        ContextActions::new(player, vec![res_button_context_actions.toggle_help()]),
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
                            flex_shrink: 0.0,
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
                                        ContextActions::new(player, vec![])
                                    ));
                                menu_bar.spawn(MenuUiStats::bundle(player, &node_q));
                                menu_bar
                                    .spawn((
                                        MenuUiActions::bundle(player, &node_q),
                                        UiFocusCycleOrder(1),
                                        ContextActions::new(player, vec![]),
                                    ));
                                menu_bar.spawn(MenuUiDescription::bundle(player, &node_q));
                            });
                        content_pane
                            .spawn((
                                GridUi::bundle(player, &node_q),
                                ContextActions::new(player, vec![]),
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
                                            StyleTty(taffy::prelude::Style {
                                                max_size: Size {
                                                    width: Dimension::Points(40.0),
                                                    height: Dimension::Points(10.0),
                                                },
                                                size: Size {
                                                    width: points(0.0),
                                                    height: points(0.0)
                                                },
                                                ..default()
                                            }),
                                            DialogueMenu::default(),
                                            ForPlayer(player),
                                            TerminalRendering::default(),
                                            VisibilityTty(true),
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

            commands.entity(player).insert((
                NodeCursor::default(),
                CursorIsHidden::default(),
                SelectedNodePiece(node_q.grid.item_at(default())),
                SelectedAction(None),
                TelegraphedAction(None),
                AvailableActionTargets::default(),
                UiFocusBundle::default(),
                AvailableMoves::default(),
                HasNodeUi,
            ));

            res_ui_ops.request(player, MainUiOp::SwitchScreen(render_root));
        }
    }
}
