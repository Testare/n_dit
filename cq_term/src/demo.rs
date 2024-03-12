use std::fs::File;
use std::io::Write;

use bevy::ecs::schedule::common_conditions::resource_added;
use bevy::ecs::system::SystemState;
use bevy::prelude::AppTypeRegistry;
use bevy::scene::DynamicSceneBuilder;
use bevy_yarnspinner::prelude::{DialogueRunner, OptionId, YarnProject, YarnSpinnerSystemSet};
use game_core::bam::BamHandle;
use game_core::board::{Board, BoardPiece, BoardPosition, BoardSize, SimplePieceInfo};
use game_core::card::{Card, Deck, Description};
use game_core::configuration::{NodeConfiguration, PlayerConfiguration};
use game_core::dialogue::Dialogue;
use game_core::node::{ForNode, IsReadyToGo, Node, NodeId, NodeOp, PlayedCards};
use game_core::op::OpResult;
use game_core::player::{ForPlayer, Player, PlayerBundle};
use game_core::prelude::*;
use game_core::quest::QuestStatus;

use crate::board_ui::{BoardBackground, BoardUi, InfoPanel, SelectedBoardPieceUi};
use crate::input_event::{KeyCode, MouseEventTty};
use crate::layout::{CalculatedSizeTty, StyleTty};
use crate::main_ui::{self, MainUi, MainUiOp, UiOps};
use crate::nf::{NFNode, NFShop, NfPlugin, RequiredNodes};
use crate::node_ui::NodeUiScreen;
use crate::prelude::KeyEvent;
use crate::render::TerminalRendering;
use crate::{KeyMap, Submap};

/// Plugin to set up temporary entities and systems while I get the game set up
#[derive(Debug)]
pub struct DemoPlugin;

#[derive(Debug, Resource, Deref, DerefMut)]
pub struct DemoNodeId(pub Option<NodeId>);

#[derive(Clone, Component, Debug)]
pub struct DebugEntityMarker;

#[derive(Debug, Default, Resource)]
pub struct DemoState {
    node_ui_id: Option<Entity>,
    board_ui_id: Option<Entity>,
    player_id: Option<Entity>,
    dialogue_runner: Option<Entity>,
}

impl Plugin for DemoPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DemoState>()
            .add_plugins(NfPlugin)
            .add_systems(
                Startup,
                demo_startup.after(main_ui::sys_startup_create_main_ui),
            )
            .add_systems(
                Update,
                demo_dialogue_runner
                    .run_if(resource_added::<YarnProject>)
                    .before(YarnSpinnerSystemSet),
            )
            .add_systems(PostUpdate, (debug_key, save_key, log_op_results));
    }
}

fn yarn_get_player_name(_: In<()>, mut q_dialogue_runner: Query<&mut DialogueRunner>) {
    for mut dialogue_runner in q_dialogue_runner.iter_mut() {
        dialogue_runner
            .variable_storage_mut()
            .set("$player_name".to_string(), "GORNATHAN".into())
            .unwrap();
    }
}

fn demo_dialogue_runner(
    mut commands: Commands,
    res_yarn: Res<YarnProject>,
    mut res_demo_state: ResMut<DemoState>,
) {
    if res_demo_state.dialogue_runner.is_none() {
        let mut dialogue_runner = res_yarn.create_dialogue_runner();
        // dialogue_runner.start_node("warez_0");
        dialogue_runner.start_node("pharmhaus_0_pr_database");
        dialogue_runner
            .commands_mut()
            .add_command("get_player_name", yarn_get_player_name);
        let dialogue_runner_id = commands
            .entity(res_demo_state.player_id.unwrap())
            .insert((Dialogue::default(), dialogue_runner))
            .id();
        // let dialogue_runner_id = commands.spawn((Dialogue::default(), dialogue_runner)).id();

        res_demo_state.dialogue_runner = Some(dialogue_runner_id);
    }
}

fn log_op_results(mut node_ops: EventReader<OpResult<NodeOp>>) {
    for op in node_ops.read() {
        log::debug!("NODE_OP_RESULT {:?}", op)
    }
}

fn save_key(world: &mut World, mut state: Local<SystemState<EventReader<KeyEvent>>>) {
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

    let type_registry = world.resource::<AppTypeRegistry>().clone();
    let entities: Vec<Entity> = world
        .query_filtered::<Entity, Or<(With<Node>, With<StyleTty>)>>()
        .iter(world)
        .collect();
    let scene = DynamicSceneBuilder::from_world(world)
        .deny_all_resources()
        .allow_all()
        .allow::<Node>()
        .extract_entities(entities.into_iter())
        .build();
    match scene.serialize_ron(&type_registry) {
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
    mut evr_mouse: EventReader<MouseEventTty>,
    mut res_demo_state: ResMut<DemoState>,
    mut res_ui_ops: ResMut<UiOps>,
    mut ev_keys: EventReader<KeyEvent>,
    mut quest_status: Query<&mut QuestStatus>,
    mut key_maps: Query<&mut KeyMap>,
    q_player_node_ui: Query<(Entity, &ForPlayer), With<NodeUiScreen>>,
    mut q_player_dr: Query<(&mut DialogueRunner, &Dialogue), With<Player>>,
    q_main_ui: Query<&MainUi>,
) {
    for layout_event in evr_mouse.read() {
        log::trace!("MOUSE EVENT: {:?}", layout_event);
    }
    for KeyEvent { code, .. } in ev_keys.read() {
        if *code == KeyCode::Char('/') {
            for mut quest_status in quest_status.iter_mut() {
                if let Some(nid) = [
                    NodeId::new("node:demo", 0),
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
        } else if *code == KeyCode::Char('8') {
            for (mut player_dr, dialogue) in q_player_dr.iter_mut() {
                if dialogue.options().is_empty() {
                    player_dr.continue_in_next_update();
                } else {
                    player_dr
                        .select_option(OptionId(0))
                        .expect("I shouldn't use this long term");
                }
            }
        } else if *code == KeyCode::Char('9') {
            for (mut player_dr, _) in q_player_dr.iter_mut() {
                player_dr
                    .select_option(OptionId(1))
                    .expect("I shouldn't use this long term");
            }
        } else if *code == KeyCode::Char('p') {
            log::debug!("Testing launching aseprite process. Later this functionality will be used to share images when the terminal doesn't support it.");
            std::process::Command::new("aseprite").spawn().unwrap();
        } else if *code == KeyCode::Char('m') {
            // TODO Better keymap logic
            for mut key_map in key_maps.iter_mut() {
                key_map.toggle_submap(Submap::Node);
            }

            q_main_ui.get_single().ok().and_then(|main_ui| {
                if res_demo_state.node_ui_id.is_none() {
                    for (node_ui_id, &ForPlayer(player_id)) in q_player_node_ui.iter() {
                        if Some(player_id) == res_demo_state.player_id {
                            res_demo_state.node_ui_id = Some(node_ui_id);
                        }
                    }
                }
                let next_screen = if **main_ui == res_demo_state.node_ui_id {
                    res_demo_state.board_ui_id
                } else {
                    res_demo_state.node_ui_id
                }?;

                res_ui_ops.request(
                    res_demo_state.player_id?,
                    MainUiOp::SwitchScreen(next_screen),
                );
                Some(())
            });
        }
    }
}

fn demo_startup(
    mut res_ui_ops: ResMut<UiOps>,
    asset_server: Res<AssetServer>,
    mut res_demo_state: ResMut<DemoState>,
    mut commands: Commands,
) {
    commands.spawn(BamHandle(asset_server.load("base.bam.txt")));
    // Create things node still needs but can't load yet

    let hack = commands
        .spawn((Card::new(
            "Hack",
            "curio:hack",
            None,
            asset_server.load("nightfall/lvl1.cards.json#Hack"),
        ),))
        .id();

    // Create player things
    let card_0 = commands
        .spawn((Card::new(
            "Slingshot",
            "curio:sling",
            None,
            asset_server.load("nightfall/lvl1.cards.json#Slingshot"),
        ),))
        .id();
    let card_1 = commands
        .spawn((Card::new(
            "Bit Man",
            "curio:bit_man",
            None,
            asset_server.load("nightfall/lvl1.cards.json#Bit Man"),
        ),))
        .id();
    let card_2 = commands
        .spawn((Card::new(
            "Bug",
            "curio:bug",
            None,
            asset_server.load("nightfall/lvl1.cards.json#Bug"),
        ),))
        .id();
    let card_3 = commands
        .spawn((Card::new(
            "Mandelbug",
            "curio:death",
            None,
            asset_server.load("nightfall/lvl3.cards.json#Mandelbug"),
        ),))
        .id();
    let card_4 = commands
        .spawn((
            Card::new(
                "Hack 3.0",
                "curio:hack",
                None,
                asset_server.load("nightfall/lvl3.cards.json#Hack 3.0"),
            ),
            Description::new("Basic attack program4"),
        ))
        .id();
    let card_5 = commands
        .spawn((Card::new(
            "Data Doctor Pro",
            "curio:data_doctor_pro",
            Some("DataDocPro"),
            asset_server.load("nightfall/lvl3.cards.json#Data Doctor Pro"),
        ),))
        .id();
    let card_bb = commands
        .spawn((Card::new(
            "Buzzbomb",
            "curio:buzzbomb",
            None,
            asset_server.load("nightfall/lvl3.cards.json#Buzzbomb"),
        ),))
        .id();
    let card_fiddle = commands
        .spawn((Card::new(
            "Fiddle",
            "curio:fiddle",
            None,
            asset_server.load("nightfall/lvl3.cards.json#Fiddle"),
        ),))
        .id();
    let card_memory_hog = commands
        .spawn((Card::new(
            "Memory Hog",
            "curio:memory_hog",
            None,
            asset_server.load("nightfall/lvl3.cards.json#Memory Hog"),
        ),))
        .id();

    let quest_status = QuestStatus::default();

    let player = commands
        .spawn((
            Name::new("Steve"),
            PlayerBundle::default(),
            PlayerConfiguration {
                node: Some(NodeConfiguration {
                    end_turn_after_all_pieces_tap: true,
                }),
            },
            quest_status,
            KeyMap::default(),
            SelectedBoardPieceUi::default(),
            PlayedCards::default(),
            IsReadyToGo(false),
            Deck::new()
                .with_card(hack)
                .with_card(hack)
                .with_card(card_0)
                .with_card(card_1)
                .with_card(card_2)
                .with_card(card_fiddle)
                // Comment the following out when demonstrating to others
                .with_card(card_bb)
                .with_card(card_3)
                .with_card(card_4)
                .with_card(card_memory_hog)
                .with_card(card_5),
        ))
        .id();

    // World map things

    let board = commands
        .spawn((Board("Network Map".into()),))
        .with_children(|board| {
            board.spawn((
                NFNode,
                ForNode(NodeId::new("node:demo", 0)),
                SimplePieceInfo("Demo Node - This node is just a testing ground".to_string()),
                TerminalRendering::default(),
                BoardPosition(Vec2 { x: 12.0, y: 25.0 }),
                BoardPiece("Smart HQ".to_owned()),
                BoardSize(Vec2 { x: 4.0, y: 1.0 }),
                Name::new("Board piece 1"),
            ));
            board.spawn((
                NFNode,
                SimplePieceInfo("Smart HQ\n".to_string()),
                ForNode(NodeId::new("node:tutorial", 0)),
                RequiredNodes(vec![NodeId::new("node:demo", 0)]),
                TerminalRendering::default(),
                BoardPiece("Smart HQ".to_owned()),
                BoardPosition(Vec2 { x: 24.0, y: 25.0 }),
                BoardSize(Vec2 { x: 4.0, y: 1.0 }),
                Name::new("Tutorial Node"),
            ));
            board.spawn((
                NFNode,
                NFShop("warez:0".to_string()),
                SimplePieceInfo("Warez Node\nLeo's Shop\nA quality shop of basic programs at low prices. Come and see what we've got to offer".to_string()),
                RequiredNodes(vec![NodeId::new("node:demo", 0)]),
                TerminalRendering::default(),
                BoardPiece("Warez".to_owned()),
                BoardPosition(Vec2 { x: 4.0, y: 20.0 }),
                BoardSize(Vec2 { x: 4.0, y: 1.0 }),
                Name::new("Warez Node: Leo's Shop"),
            ));
            board.spawn((
                NFNode,
                SimplePieceInfo("Pharmaus\nPR Database\nSecurity Level: 1".to_string()),
                ForNode(NodeId::new("node:area1", 0)),
                RequiredNodes(vec![NodeId::new("node:tutorial", 0)]),
                TerminalRendering::default(),
                BoardPiece("Pharmhaus".to_owned()),
                BoardPosition(Vec2 { x: 28.0, y: 22.0 }),
                BoardSize(Vec2 { x: 4.0, y: 1.0 }),
                Name::new("Pharmaus: PR Database"),
            ));
            board.spawn((
                NFNode,
                SimplePieceInfo("Lucky Monkey Media\nTech Support\nSecurity Level: 1".to_string()),
                ForNode(NodeId::new("node:area1", 1)),
                RequiredNodes(vec![NodeId::new("node:tutorial", 0)]),
                TerminalRendering::default(),
                BoardPiece("Lucky Monkey".to_owned()),
                BoardPosition(Vec2 { x: 28.0, y: 30.0 }),
                BoardSize(Vec2 { x: 4.0, y: 1.0 }),
                Name::new("Lucky Monkey: Tech Support"),
            ));
        })
        .id();

    let board_ui_root = commands
        .spawn((
            Name::new("Network map"),
            TerminalRendering::new(Vec::new()),
            CalculatedSizeTty(UVec2 { x: 400, y: 500 }),
            StyleTty(taffy::style::Style {
                grid_row: taffy::prelude::line(1),
                grid_column: taffy::prelude::line(1),
                flex_direction: taffy::style::FlexDirection::Column,
                ..default()
            }),
        ))
        .with_children(|board_ui_root| {
            use taffy::prelude::*;
            board_ui_root.spawn((
                Name::new("Network map title bar"),
                ForPlayer(player),
                StyleTty(taffy::style::Style {
                    size: Size {
                        width: Dimension::Auto,
                        height: Dimension::Points(2.),
                    },
                    padding: Rect {
                        bottom: LengthPercentage::Points(1.0),
                        ..TaffyZero::ZERO
                    },
                    max_size: Size {
                        width: Dimension::Points(100.0),
                        height: Dimension::Auto,
                    },
                    flex_shrink: 0.0,
                    ..Default::default()
                }),
                TerminalRendering::new(vec!["Network Map".to_owned()]),
            ));
            board_ui_root
                .spawn((
                    StyleTty(taffy::prelude::Style {
                        size: Size {
                            width: Dimension::Auto,
                            height: Dimension::Auto,
                        },
                        flex_grow: 1.0,
                        flex_shrink: 0.0,
                        ..default()
                    }),
                    Name::new("Network Map Content Pane"),
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
                                StyleTty(Style::default()),
                                ForPlayer(player),
                                InfoPanel,
                                Name::new("InfoPanel"),
                                TerminalRendering::default(),
                            ));
                        });
                    content_pane.spawn((
                        Name::new("Demo map background"),
                        ForPlayer(player),
                        BoardUi(board),
                        BoardBackground(asset_server.load("nightfall/net_map.charmi.toml")),
                        StyleTty(taffy::style::Style {
                            display: taffy::style::Display::Grid,
                            // grid_template_columns: 18 x 5 for now
                            grid_auto_rows: vec![NonRepeatedTrackSizingFunction {
                                min: taffy::style::MinTrackSizingFunction::Fixed(
                                    taffy::style::LengthPercentage::Points(1.0),
                                ),
                                max: taffy::style::MaxTrackSizingFunction::Fixed(
                                    taffy::style::LengthPercentage::Points(1.0),
                                ),
                            }],
                            grid_auto_columns: vec![NonRepeatedTrackSizingFunction {
                                min: taffy::style::MinTrackSizingFunction::Fixed(
                                    taffy::style::LengthPercentage::Points(1.0),
                                ),
                                max: taffy::style::MaxTrackSizingFunction::Fixed(
                                    taffy::style::LengthPercentage::Points(1.0),
                                ),
                            }],
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
