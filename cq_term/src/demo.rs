use std::fs::File;
use std::io::Write;

use bevy::audio::Volume;
use bevy::ecs::system::SystemState;
use bevy::hierarchy::ChildBuilder;
use bevy::scene::DynamicSceneBuilder;
use charmi::CharacterMapImage;
use crossterm::style::{ContentStyle, Stylize};
use game_core::bam::BamHandle;
use game_core::board::{Board, BoardPiece, BoardPosition, BoardScreen, BoardSize, SimplePieceInfo};
use game_core::card::{CardDefinition, CardHandle, Deck, Nickname};
use game_core::configuration::{NodeConfiguration, PlayerConfiguration};
use game_core::dialog::Dialog;
use game_core::item::{Item, ItemOp, Wallet};
use game_core::node::{
    ForNode, InNode, Node, NodeId, NodeOp, OnTeam, PlayedCards, Team, TeamStatus,
};
use game_core::op::{CoreOps, OpResult};
use game_core::player::{ForPlayer, Ncp, Player, PlayerBundle};
use game_core::prelude::*;
use game_core::quest::QuestStatus;
use game_core::saving::SaveOp;
use game_core::shop::{ShopId, ShopInventory, ShopListing, ShopOp};

use crate::animation::AnimationPlayer;
use crate::base_ui::context_menu::ContextActions;
use crate::base_ui::{
    ButtonUiBundle, FlexibleTextUi, FlexibleTextUiMultiline, HoverPoint, PopupMenu, Tooltip,
};
use crate::board_ui::{ActionsPanel, BoardBackground, BoardUi, InfoPanel, SelectedBoardPieceUi};
use crate::configuration::DrawConfiguration;
use crate::dialog_ui::{DialogLineUi, DialogOptionUi, DialogUiContextActions};
use crate::input_event::{KeyCode, MouseEventListener, MouseEventTty};
use crate::layout::{CalculatedSizeTty, StyleTty, VisibilityTty};
use crate::main_ui::{
    self, HudContextActions, ItemDetailsUiActions, ItemDetailsUiDescription, ItemDetailsUiStats,
    MainUiOp, MonDisplay, SaveButton, ShopListingUi, ShopNotification, ShopUi, ShopUiBuyButton,
    ShopUiFinishShoppingButton, ShopUiSelectedItem, UiOps,
};
use crate::nf::{NFNode, NFShop, NfPlugin, RequiredNodes, VictoryDialogue};
use crate::prelude::KeyEvent;
use crate::render::TerminalRendering;
use crate::KeyMap;

/// Plugin to set up temporary entities and systems while I get the game set up
#[derive(Debug)]
pub struct DemoPlugin;

#[derive(Debug, Resource, Deref, DerefMut)]
pub struct DemoNodeId(pub Option<NodeId>);

#[derive(Clone, Component, Debug)]
pub struct DebugEntityMarker;

#[derive(Debug, Default, Resource)]
pub struct DemoState {
    board_ui_id: Option<Entity>,
    player_id: Option<Entity>,
}

#[derive(Debug, Resource)]
pub struct UseDemoShader(pub u32);

#[derive(Component, Debug, Default)]
pub struct DemoShader {
    color: u8,
}

impl Plugin for DemoPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DemoState>()
            .add_plugins(NfPlugin)
            .add_systems(
                Startup,
                demo_startup.after(main_ui::sys_startup_create_main_ui),
            )
            .add_systems(Update, sys_demo_shader)
            .add_systems(PostUpdate, (debug_key, dump_key, log_op_results));
    }
}

pub fn sys_demo_shader(
    mut q_demo_shader: Query<(&CalculatedSizeTty, &mut DemoShader, &mut TerminalRendering)>,
) {
    use crossterm::style::*;
    for (size, mut ds, mut tr) in q_demo_shader.iter_mut() {
        let mut charmi = CharacterMapImage::new();
        for y in 0..size.height() {
            let row = charmi.new_row();
            for cell_style in (0..size.width()).map(|x| {
                let color_val = ds.color.wrapping_add(((x + y) % 256) as u8);

                ContentStyle::new()
                    .on(Color::AnsiValue(color_val))
                    .with(Color::AnsiValue(color_val)) // Remove or change this if you want to see rainbow
                                                       // affect applied OVER game text
            }) {
                row.add_effect(1, &cell_style);
            }
        }
        tr.update_charmie(charmi);
        ds.color = ds.color.wrapping_add(1);
    }
}

fn log_op_results(
    mut evr_node_op: EventReader<OpResult<NodeOp>>,
    mut evr_item_op: EventReader<OpResult<ItemOp>>,
    mut evr_shop_op: EventReader<OpResult<ShopOp>>,
    mut evr_save_op: EventReader<OpResult<SaveOp>>,
) {
    for op in evr_node_op.read() {
        log::debug!("NodeOp Result: {:?}", op)
    }
    for op in evr_item_op.read() {
        log::debug!("ItemOp Result: {:?}", op)
    }
    for op in evr_shop_op.read() {
        log::debug!("ShopOp Result: {:?}", op)
    }
    for op in evr_save_op.read() {
        log::debug!("SaveOp Result: {:?}", op)
    }
}

fn dump_key(world: &mut World, mut state: Local<SystemState<EventReader<KeyEvent>>>) {
    let mut evr_keys = state.get(world);
    let save_button_pressed = evr_keys.read().any(|event| {
        matches!(
            event,
            KeyEvent {
                code: KeyCode::Char('*'),
                ..
            }
        )
    });
    if !save_button_pressed {
        return;
    }

    let app_type_registry = world.resource::<AppTypeRegistry>().clone();
    let entities: Vec<Entity> = world
        .query_filtered::<Entity, Or<(With<Player>, With<Node>)>>()
        .iter(world)
        .collect();
    let scene = DynamicSceneBuilder::from_world(world)
        .deny_all_resources()
        .allow_all()
        .allow::<Node>()
        .extract_entities(entities.into_iter())
        .build();
    let type_registry = app_type_registry.read();
    match scene.serialize(&type_registry) {
        Ok(scene_serialized) => {
            log::info!("Serialization successful");
            File::create("debug.scn.ron")
                .and_then(|mut file| file.write(scene_serialized.as_bytes()))
                .expect("Error occured writing file");
        },
        Err(err) => {
            log::info!("Serialization NOT successful: {:?}", err);
        },
    }
}

fn debug_key(
    mut commands: Commands,
    mut res_core_ops: ResMut<CoreOps>,
    mut evr_mouse: EventReader<MouseEventTty>,
    mut ev_keys: EventReader<KeyEvent>,
    mut q_quest_status: Query<&mut QuestStatus>,
    q_player_ncp: Query<(Entity, Option<&OnTeam>, &InNode), (With<Player>, With<Ncp>)>,
    q_node: Query<(Entity, &TeamStatus), With<Node>>,
    q_team: Query<Entity, With<Team>>,
) {
    for layout_event in evr_mouse.read() {
        log::trace!("MOUSE EVENT: {:?}", layout_event);
    }
    for KeyEvent { code, .. } in ev_keys.read() {
        if *code == KeyCode::Char('/') {
            for mut quest_status in q_quest_status.iter_mut() {
                if let Some(nid) = [
                    NodeId::new("node:tutorial", 0),
                    NodeId::new("node:area1", 0),
                    NodeId::new("node:area1", 1),
                ]
                .iter()
                .find(|&nid| !quest_status.is_node_done(nid))
                {
                    quest_status.record_node_done(nid);
                }
            }
        } else if *code == KeyCode::Char('p') {
            log::debug!("Testing launching aseprite process. Later this functionality will be used to share images when the terminal doesn't support it.");
            std::process::Command::new("aseprite").spawn().unwrap();
        } else if *code == KeyCode::Char('i') {
            for (player_id, on_team, in_node) in q_player_ncp.iter() {
                log::debug!("TEAM for [{player_id:?}]: {on_team:?} / Node {in_node:?}");
            }
            for (node_id, team_status) in q_node.iter() {
                log::debug!("TEAM STATUS for [{node_id:?}]: {team_status:?}");
            }
            for team_id in q_team.iter() {
                commands.entity(team_id).log_components();
            }
        } else if *code == KeyCode::Char('9') {
            res_core_ops.request(Entity::PLACEHOLDER, SaveOp::Load);
        }
    }
}

fn demo_startup(
    mut res_ui_ops: ResMut<UiOps>,
    mut res_core_ops: ResMut<CoreOps>,
    res_use_demo_shader: Res<UseDemoShader>,
    res_draw_config: Res<DrawConfiguration>,
    res_dialog_context_actions: Res<DialogUiContextActions>,
    res_hud_context_actions: Res<HudContextActions>,
    asset_server: Res<AssetServer>,
    mut res_demo_state: ResMut<DemoState>,
    mut commands: Commands,
) {
    commands.spawn(BamHandle(asset_server.load("base.bam.txt")));

    let stabby_boi = commands
        .spawn((
            asset_server.load::<CardDefinition>("nightfall/lvl1.cards.json#Hack"),
            CardHandle("nightfall/lvl1.cards.json#Hack".to_string()),
            Nickname::new("Stabby boi"),
        ))
        .id();

    // TODO FIXME This is to reduce crashes from bevy issue
    // SEE https://github.com/bevyengine/bevy/issues/10820
    // When issue is resolved, remove these
    commands.spawn(asset_server.load::<()>("nightfall/lvl1.cards.json"));
    commands.spawn(asset_server.load::<()>("nightfall/lvl2.cards.json"));
    commands.spawn(asset_server.load::<()>("nightfall/lvl3.cards.json"));
    commands.spawn(asset_server.load::<()>("nightfall/lvl4.cards.json"));
    commands.spawn(asset_server.load::<()>("nightfall/enemies.cards.json"));

    // This is to fix bug where game "crashes" (spams console)
    // if no audio is playing, we are adding "background music" of
    // silence.
    // See issue: https://github.com/bevyengine/bevy/issues/9798
    // NOTE: This unfortunately did not work when playing a silent audio file, trying a real audio file with 0 volume
    // NOTE: Yeah it didn't work with music either.
    commands.spawn(AudioBundle {
        source: asset_server.load("tmp/audio/mixkit-coins-sound-2003.wav"),
        settings: PlaybackSettings {
            mode: bevy::audio::PlaybackMode::Loop,
            volume: Volume::new(0.0),
            ..default()
        },
    });

    // Add demo cards
    let card_def_paths = [
        "nightfall/lvl1.cards.json#Slingshot",
        "nightfall/lvl1.cards.json#Bit Man",
        "nightfall/lvl1.cards.json#Bug",
        "nightfall/lvl3.cards.json#Mandelbug",
        "nightfall/lvl3.cards.json#Hack 3.0",
        "nightfall/lvl3.cards.json#Data Doctor Pro",
        "nightfall/lvl3.cards.json#Buzzbomb",
        "nightfall/lvl3.cards.json#Fiddle",
        "nightfall/lvl3.cards.json#Memory Hog",
    ];

    // World map things

    let board_size = Vec2 { x: 93.0, y: 38.0 };
    let board = commands
        .spawn((Board("Network Map".into()), BoardSize(board_size)))
        .with_children(|board| {
            board.spawn((
                NFNode,
                ForNode(NodeId::new("node:demo", 0)),
                SimplePieceInfo("Demo Node - This node is just a testing ground".to_string()),
                BoardPosition(Vec2 { x: 12.0, y: 25.0 }),
                BoardPiece("Smart HQ".to_owned()),
                BoardSize(Vec2 { x: 4.0, y: 1.0 }),
                Name::new("Board piece 1"),
            ));
            board.spawn((
                BoardPiece("Smart HQ".to_owned()),
                BoardPosition(Vec2 { x: 24.0, y: 25.0 }),
                BoardSize(Vec2 { x: 4.0, y: 1.0 }),
                ForNode(NodeId::new("node:tutorial", 0)),
                Name::new("Tutorial Node"),
                NFNode,
                SimplePieceInfo("Smart HQ\n".to_string()),
            ));
            board.spawn((
                BoardPiece("Warez".to_owned()),
                BoardPosition(Vec2 { x: 4.0, y: 20.0 }),
                BoardSize(Vec2 { x: 4.0, y: 1.0 }),
                Name::new("Warez Node: Leo's Shop"),
                NFNode,
                NFShop("warez:0".to_string()),
                ShopId(SetId::new_unchecked("warez", 0)),
                RequiredNodes(vec![NodeId::new("node:tutorial", 0)]),
                SimplePieceInfo("Warez Node\nLeo's Shop\nA quality shop of basic programs at low prices. Come and see what we've got to offer".to_string()),
                ShopInventory(vec![
                    ShopListing::new(500, Item::Card(asset_server.load("nightfall/lvl1.cards.json#Hack"))),
                    ShopListing::new(750, Item::Card(asset_server.load("nightfall/lvl1.cards.json#Bug"))),
                    ShopListing::new(750, Item::Card(asset_server.load("nightfall/lvl1.cards.json#Slingshot"))),
                    ShopListing::new(500, Item::Card(asset_server.load("nightfall/lvl1.cards.json#Data Doctor"))),
                    ShopListing::new(250, Item::Card(asset_server.load("nightfall/lvl1.cards.json#Bit Man"))),
                ]),
            ));
            board.spawn((
                BoardPiece("Pharmhaus".to_owned()),
                BoardPosition(Vec2 { x: 28.0, y: 22.0 }),
                BoardSize(Vec2 { x: 4.0, y: 1.0 }),
                ForNode(NodeId::new("node:area1", 0)),
                Name::new("Pharmaus: PR Database"),
                NFNode,
                RequiredNodes(vec![NodeId::new("node:tutorial", 0)]),
                SimplePieceInfo("Pharmaus\nPR Database\nSecurity Level: 1".to_string()),
                VictoryDialogue::new("pharmhaus_0_pr_database"),
            ));
            board.spawn((
                BoardPiece("Lucky Monkey".to_owned()),
                BoardPosition(Vec2 { x: 28.0, y: 30.0 }),
                BoardSize(Vec2 { x: 4.0, y: 1.0 }),
                ForNode(NodeId::new("node:area1", 1)),
                Name::new("Lucky Monkey: Tech Support"),
                NFNode,
                RequiredNodes(vec![NodeId::new("node:tutorial", 0)]),
                SimplePieceInfo("Lucky Monkey Media\nTech Support\nSecurity Level: 1".to_string()),
                VictoryDialogue::new("lucky_monkey_0_tech_support"),
            ));
        })
        .id();

    let player = commands
        .spawn((
            Deck::new().with_card(stabby_boi),
            Dialog::default(),
            KeyMap::default(),
            Name::new("Steve"),
            Ncp,
            PlayedCards::default(),
            PlayerBundle::default(),
            PlayerConfiguration {
                node: Some(NodeConfiguration {
                    end_turn_after_all_pieces_tap: true,
                }),
            },
            QuestStatus::default(),
            SelectedBoardPieceUi::default(),
            Wallet::new().with_mon(10_000), // Just for demo
        ))
        .id();

    for card_def_path in card_def_paths.into_iter() {
        res_core_ops.request(
            player,
            ItemOp::AddItem {
                item: Item::Card(asset_server.load(card_def_path)),
                refund: 0,
            },
        );
    }

    let board_ui_root = commands
        .spawn((
            BoardScreen,
            CalculatedSizeTty(UVec2 { x: 400, y: 500 }),
            ForPlayer(player),
            Name::new("Network map"),
            StyleTty(taffy::style::Style {
                grid_row: taffy::prelude::line(1),
                grid_column: taffy::prelude::line(1),
                flex_direction: taffy::style::FlexDirection::Column,
                ..default()
            }),
            TerminalRendering::new(Vec::new()),
        ))
        .with_children(|board_ui_root| {
            use taffy::prelude::*;
            board_ui_root
                .spawn((
                    Name::new("Network map title bar"),
                    ForPlayer(player),
                    StyleTty(taffy::style::Style {
                        size: Size {
                            width: Dimension::Auto,
                            height: length(2.0),
                        },
                        padding: Rect {
                            bottom: length(1.0),
                            ..TaffyZero::ZERO
                        },
                        flex_direction: FlexDirection::Row,
                        flex_shrink: 0.0,
                        ..Default::default()
                    }),
                ))
                .with_children(|title_bar| {
                    if res_use_demo_shader.0 > 0 {
                        title_bar.spawn((
                            Name::new("Demo shader"),
                            StyleTty(taffy::style::Style {
                                size: Size {
                                    width: length(256.0),
                                    height: length(res_use_demo_shader.0 as f32),
                                },
                                flex_grow: 1.0,
                                ..Default::default()
                            }),
                            DemoShader::default(),
                            TerminalRendering::default(),
                        ));
                    } else {
                        title_bar.spawn((
                            Name::new("Network map title"),
                            ForPlayer(player),
                            StyleTty(taffy::style::Style {
                                size: Size {
                                    width: length(12.0),
                                    height: length(1.0),
                                },
                                padding: Rect {
                                    bottom: length(1.0),
                                    ..TaffyZero::ZERO
                                },
                                flex_direction: FlexDirection::Row,
                                flex_grow: 1.0,
                                flex_shrink: 0.0,
                                ..Default::default()
                            }),
                            TerminalRendering::new(vec!["Network Map".to_owned()]),
                        ));
                        title_bar.spawn((
                            Name::new("Mon Display"),
                            MonDisplay::default(),
                            ForPlayer(player),
                            StyleTty(taffy::style::Style {
                                size: Size {
                                    width: length(10.0),
                                    height: length(1.0),
                                },
                                padding: Rect {
                                    bottom: length(1.0),
                                    ..TaffyZero::ZERO
                                },
                                flex_direction: FlexDirection::Row,
                                flex_grow: 1.0,
                                flex_shrink: 0.0,
                                ..Default::default()
                            }),
                            TerminalRendering::new(vec!["$Mon Display".to_owned()]),
                        ));

                        title_bar.spawn((
                            ForPlayer(player),
                            SaveButton,
                            ButtonUiBundle::new("Save", ContentStyle::new().green()),
                            ContextActions::new(player, &[res_hud_context_actions.save()]),
                            // ContextActions::new(player, &[res_button_context_actions.undo()]),
                            // VisibilityTty::invisible(),
                            /*StyleTty(taffy::style::Style {
                                size: Size {
                                    width: length(.0),
                                    height: length(1.0),
                                },
                                padding: Rect {
                                    bottom: length(1.0),
                                    ..TaffyZero::ZERO
                                },
                                flex_direction: FlexDirection::Row,
                                flex_grow: 1.0,
                                flex_shrink: 0.0,
                                ..Default::default()
                            }),*/
                            Tooltip::new("Save"),
                        ));
                    }
                });
            board_ui_root
                .spawn((
                    StyleTty(taffy::prelude::Style {
                        size: Size {
                            width: Dimension::Auto,
                            height: Dimension::Auto,
                        },
                        flex_grow: 1.0,
                        flex_shrink: 0.0,
                        display: taffy::style::Display::Grid,
                        grid_template_columns: vec![
                            length(14.0),
                            minmax(length(10.0), fr(1.0)),
                            fr(0.0000001),
                        ],
                        grid_template_rows: vec![fr(1.0), fr(0.000001)],
                        ..default()
                    }),
                    Name::new("Network Map Content Pane"),
                ))
                .with_children(|content_pane| {
                    content_pane
                        .spawn((
                            StyleTty(taffy::prelude::Style {
                                grid_row: line(1),
                                grid_column: line(1),
                                flex_direction: FlexDirection::Column,
                                display: Display::Flex,
                                ..default()
                            }),
                            Name::new("Board Menu Bar"),
                        ))
                        .with_children(|menu_bar| {
                            menu_bar.spawn((
                                StyleTty(Style::default()),
                                ForPlayer(player),
                                InfoPanel,
                                Name::new("Info Panel"),
                                TerminalRendering::default(),
                            ));
                            menu_bar.spawn((
                                StyleTty(Style {
                                    flex_direction: FlexDirection::Column,
                                    ..Style::default()
                                }),
                                ForPlayer(player),
                                ActionsPanel,
                                Name::new("Actions Panel"),
                                TerminalRendering::default(),
                            ));
                        });
                    content_pane
                        .spawn((
                            Name::new("Board UI Popup menu pane"),
                            StyleTty(Style {
                                display: Display::Grid,
                                grid_row: line(1),
                                grid_column: line(2),
                                max_size: Size {
                                    width: length(board_size.x),
                                    height: length(board_size.y),
                                },
                                grid_template_columns: vec![
                                    fr(1.0),
                                    minmax(length(0.0), max_content()),
                                    fr(1.0),
                                ],
                                grid_template_rows: vec![
                                    fr(1.0),
                                    minmax(length(0.0), max_content()),
                                    fr(1.0),
                                ],
                                ..default()
                            }),
                        ))
                        .with_children(|popup_menu_pane| {
                            build_popup_menu(
                                res_draw_config,
                                player,
                                res_dialog_context_actions.say_this(),
                                popup_menu_pane,
                            );
                        });
                    content_pane.spawn((
                        Name::new("Board background"),
                        ForPlayer(player),
                        BoardUi(board),
                        BoardBackground(asset_server.load("nightfall/net_map.charmi.toml")),
                        CalculatedSizeTty::default(),
                        StyleTty(taffy::style::Style {
                            display: taffy::style::Display::Grid,
                            max_size: Size {
                                width: length(board_size.x),
                                height: length(board_size.y),
                            },
                            grid_row: line(1),
                            grid_column: line(2),
                            grid_template_rows: vec![repeat(
                                GridTrackRepetition::AutoFill,
                                vec![length(1.0)],
                            )],
                            grid_template_columns: vec![repeat(
                                GridTrackRepetition::AutoFill,
                                vec![length(1.0)],
                            )],
                            ..default()
                        }),
                        TerminalRendering::new(Vec::new()),
                    ));
                });
        })
        .id();

    // Demo logic things
    res_demo_state.board_ui_id = Some(board_ui_root);
    res_demo_state.player_id = Some(player);
    res_ui_ops.request(player, MainUiOp::SwitchScreen(board_ui_root));
}

pub fn build_popup_menu(
    res_draw_config: Res<DrawConfiguration>,
    player: Entity,
    say_this_ca: Entity,
    popup_menu_pane: &mut ChildBuilder,
) {
    use taffy::prelude::*;
    popup_menu_pane
        .spawn((
            TerminalRendering::default(),
            Name::new("Node popup menu"),
            StyleTty(taffy::prelude::Style {
                flex_grow: 0.0,
                grid_row: line(2),
                grid_column: line(2),
                padding: length(1.0),
                flex_direction: FlexDirection::Column,
                ..default()
            }),
            MouseEventListener, // To prevent grid from interacting
            PopupMenu,
        ))
        .with_children(|popup_menu| {
            popup_menu.spawn((
                StyleTty(taffy::prelude::Style {
                    max_size: Size {
                        width: length(40.0),
                        height: length(8.0),
                    },
                    size: Size {
                        width: length(0.0),
                        height: length(0.0),
                    },
                    ..default()
                }),
                DialogLineUi::default(),
                ForPlayer(player),
                TerminalRendering::default(),
                VisibilityTty(true),
            ));
            popup_menu.spawn((
                StyleTty(taffy::prelude::Style {
                    max_size: Size {
                        width: length(40.0),
                        height: length(4.0),
                    },
                    size: zero(),
                    ..default()
                }),
                HoverPoint::default(),
                DialogOptionUi(0),
                ContextActions::new(player, &[say_this_ca]),
                MouseEventListener,
                ForPlayer(player),
                TerminalRendering::default(),
                VisibilityTty(true),
            ));
            popup_menu.spawn((
                StyleTty(taffy::prelude::Style {
                    max_size: Size {
                        width: length(40.0),
                        height: length(4.0),
                    },
                    size: zero(),
                    ..default()
                }),
                HoverPoint::default(),
                DialogOptionUi(1),
                ContextActions::new(player, &[say_this_ca]),
                MouseEventListener,
                ForPlayer(player),
                TerminalRendering::default(),
                VisibilityTty(true),
            ));
            popup_menu.spawn((
                StyleTty(taffy::prelude::Style {
                    max_size: Size {
                        width: length(40.0),
                        height: length(4.0),
                    },
                    size: zero(),
                    ..default()
                }),
                HoverPoint::default(),
                DialogOptionUi(2),
                ContextActions::new(player, &[say_this_ca]),
                MouseEventListener,
                ForPlayer(player),
                TerminalRendering::default(),
                VisibilityTty(true),
            ));
            popup_menu
                .spawn((
                    StyleTty(taffy::prelude::Style {
                        max_size: Size {
                            width: length(40.0),
                            height: length(30.0), // Will need to implement scrolling
                        },
                        flex_direction: FlexDirection::Column,
                        ..default()
                    }),
                    ShopUi,
                    ShopUiSelectedItem::default(),
                    ForPlayer(player),
                    Name::new("Shop UI"),
                    VisibilityTty(false),
                ))
                .with_children(|shop_ui| {
                    shop_ui.spawn((
                        StyleTty(Style {
                            size: Size {
                                width: auto(),
                                height: length(1.0),
                            },
                            flex_shrink: 0.0,
                            ..default()
                        }),
                        ShopNotification,
                        TerminalRendering::default(),
                        AnimationPlayer::default(),
                        ForPlayer(player),
                    ));
                    shop_ui.spawn((
                        StyleTty(taffy::prelude::Style {
                            flex_direction: FlexDirection::Column,
                            ..default()
                        }),
                        ShopListingUi,
                        ContextActions::new(player, &[say_this_ca]),
                        ForPlayer(player),
                    ));
                    shop_ui
                        .spawn((
                            StyleTty(taffy::prelude::Style {
                                display: Display::Grid,
                                max_size: Size {
                                    height: length(1.0),
                                    width: auto(),
                                },
                                grid_template_columns: vec![fr(1.0), fr(1.0)],
                                ..default()
                            }),
                            Name::new("Shop button bar"),
                        ))
                        .with_children(|shop_button_bar| {
                            shop_button_bar.spawn((
                                ShopUiBuyButton,
                                ForPlayer(player),
                                ButtonUiBundle::new(
                                    "Buy",
                                    res_draw_config.color_scheme().shop_ui_buy_button(),
                                ),
                            ));
                            shop_button_bar.spawn((
                                ShopUiFinishShoppingButton,
                                ForPlayer(player),
                                ButtonUiBundle::new(
                                    "Done",
                                    res_draw_config.color_scheme().shop_ui_done_button(),
                                ),
                            ));
                        });
                    shop_ui.spawn((
                        StyleTty(Style {
                            size: Size {
                                width: auto(),
                                height: length(1.0),
                            },
                            margin: Rect {
                                top: length(1.0),
                                ..zero()
                            },
                            flex_shrink: 0.0,
                            ..default()
                        }),
                        ItemDetailsUiStats,
                        VisibilityTty(false),
                        FlexibleTextUi {
                            style: ContentStyle::new().magenta(),
                            text: "".to_owned(),
                        },
                        Name::new("Shop UI/Item Details/Stats"),
                        TerminalRendering::default(),
                        ForPlayer(player),
                    ));
                    shop_ui.spawn((
                        StyleTty(Style {
                            size: Size {
                                width: auto(),
                                height: length(2.0),
                            },
                            margin: Rect {
                                top: length(1.0),
                                ..zero()
                            },
                            padding: Rect {
                                top: length(1.0),
                                ..zero()
                            },
                            ..default()
                        }),
                        Name::new("Shop UI/Item Details/Actions"),
                        ItemDetailsUiActions,
                        VisibilityTty(true),
                        TerminalRendering::new(vec!["--Actions--".to_string()]),
                        ForPlayer(player),
                    ));
                    shop_ui.spawn((
                        StyleTty(Style {
                            size: Size {
                                width: auto(),
                                height: length(3.0),
                            },
                            margin: Rect {
                                top: length(1.0),
                                ..zero()
                            },
                            flex_shrink: 0.0,
                            ..default()
                        }),
                        ItemDetailsUiDescription,
                        VisibilityTty(true),
                        FlexibleTextUiMultiline {
                            style: ContentStyle::new().cyan(),
                            text: "".to_owned(),
                        },
                        Name::new("Shop UI/Item Details/Description"),
                        TerminalRendering::default(),
                        ForPlayer(player),
                    ));
                });
        });
}
