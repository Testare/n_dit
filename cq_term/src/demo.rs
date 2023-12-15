use std::fs::File;
use std::io::Write;

use bevy::ecs::system::SystemState;
use bevy::prelude::AppTypeRegistry;
use bevy::scene::{DynamicScene, DynamicSceneBuilder};
use game_core::bam::BamHandle;
use game_core::board::{Board, BoardPiece, BoardPosition, BoardSize};
use game_core::card::{Card, Deck, Description};
use game_core::configuration::{NodeConfiguration, PlayerConfiguration};
use game_core::node::{
    EnteringNode, ForNode, IsReadyToGo, NoOpAction, Node, NodeId, NodeOp, NodePiece, PlayedCards,
    Team, TeamStatus, Teams,
};
use game_core::op::OpResult;
use game_core::player::{ForPlayer, Player, PlayerBundle};
use game_core::prelude::*;
use game_core::quest::QuestStatus;

use crate::board_ui::{BoardBackground, BoardUi};
use crate::fx::Fx;
use crate::input_event::{KeyCode, MouseEventTty};
use crate::layout::{CalculatedSizeTty, LayoutRoot, StyleTty};
use crate::nf::{NFNode, NfPlugin, RequiredNodes};
use crate::node_ui::NodeCursor;
use crate::prelude::KeyEvent;
use crate::render::TerminalRendering;
use crate::{KeyMap, Submap, TerminalWindow};

/// Plugin to set up temporary entities and systems while I get the game set up
#[derive(Debug)]
pub struct DemoPlugin;

#[derive(Component, Debug)]
pub struct DebugEntityMarker;

#[derive(Debug, Default, Resource)]
pub struct DemoState {
    node_ui_id: Option<Entity>,
    board_ui_id: Option<Entity>,
    node_id: Option<Entity>,
    player_id: Option<Entity>,
}

impl Plugin for DemoPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DemoState>()
            .add_plugins(NfPlugin)
            .add_systems(Startup, demo_startup)
            .add_systems(PostUpdate, (debug_key, save_key, log_op_results));
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
        .query_filtered::<Entity, Or<(
            With<Node>,
            With<NodePiece>,
            With<Team>,
            // With<Card>,
            With<Player>,
        )>>()
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
    mut res_terminal_window: ResMut<TerminalWindow>,
    mut res_demo_state: ResMut<DemoState>,
    asset_server: Res<AssetServer>,
    fx: Res<Fx>,
    mut ev_keys: EventReader<KeyEvent>,
    mut quest_status: Query<&mut QuestStatus>,
    mut key_maps: Query<&mut KeyMap>,
    nodes: Query<
        (
            Entity,
            &EntityGrid,
            Option<&NodeCursor>,
            &Teams,
            &TeamStatus,
        ),
        With<Node>,
    >,
) {
    for layout_event in evr_mouse.read() {
        log::trace!("MOUSE EVENT: {:?}", layout_event);
    }
    for KeyEvent { code, .. } in ev_keys.read() {
        if *code == KeyCode::Char('/') {
            for mut quest_status in quest_status.iter_mut() {
                if let Some(n) =
                    (0..32).find(|i| !quest_status.is_node_done(&NodeId::new("node:demo", *i)))
                {
                    quest_status.record_node_done(&NodeId::new("node:demo", n));
                }
            }
            log::debug!("Debug event occured");
            log::debug!(
                "Pickup sound load state: {:?}",
                asset_server.get_load_state(fx.pickup_sound.clone())
            );
            log::debug!(
                "Animation load state: {:?}",
                asset_server.get_load_state(fx.charmia.clone())
            );

            for (_, entity_grid, cursor, teams, team_status) in nodes.iter() {
                log::debug!(
                    "# Node ({:?}) - Teams ({:?}) - Team Status ({:?})",
                    cursor,
                    teams,
                    team_status
                );
                for entry in entity_grid.entities() {
                    log::debug!("Entity: {:?}", entry);
                }
            }
        } else if *code == KeyCode::Char('p') {
            log::debug!("Testing launching aseprite process. Later this functionality will be used to share images when the terminal doesn't support it.");
            std::process::Command::new("aseprite").spawn().unwrap();
        } else if *code == KeyCode::Char('m') {
            let current_render_target = res_terminal_window.render_target();
            for mut key_map in key_maps.iter_mut() {
                key_map.toggle_submap(Submap::Node);
            }

            if res_demo_state.node_ui_id.is_none() {
                res_demo_state.node_ui_id = current_render_target;
            }
            if current_render_target == res_demo_state.node_ui_id {
                res_terminal_window.set_render_target(res_demo_state.board_ui_id);
            } else {
                res_terminal_window.set_render_target(res_demo_state.node_ui_id);
            }
        }
    }
}

#[allow(unused)] // While setting up map
fn demo_startup(
    asset_server: Res<AssetServer>,
    no_op: Res<NoOpAction>,
    mut res_demo_state: ResMut<DemoState>,
    mut res_terminal_window: ResMut<TerminalWindow>,
    mut commands: Commands,
) {
    let root_bam = commands.spawn(BamHandle(asset_server.load("base.bam.txt")));
    // Create things node still needs but can't load yet

    let hack = commands
        .spawn((Card::new(
            "Hack",
            "curio:hack",
            None,
            asset_server.load("nightfall/lvl1.cards.json#Hack"),
        ),))
        .id();

    // Create node things
    let demo_node_id = NodeId::new("node:tutorial", 1);
    let demo_node_id_clone = demo_node_id.clone();

    let node_asset_handle: Handle<DynamicScene> = asset_server.load("demo/demo.scn.ron");
    commands.spawn(node_asset_handle);
    let node_asset_handle: Handle<DynamicScene> =
        asset_server.load("nightfall/nodes/tutorial.scn.ron");
    commands.spawn(node_asset_handle);
    let node_asset_handle: Handle<DynamicScene> =
        asset_server.load("nightfall/nodes/node1.scn.ron");
    commands.spawn(node_asset_handle);
    let node_asset_handle: Handle<DynamicScene> =
        asset_server.load("nightfall/nodes/node2.scn.ron");
    commands.spawn(node_asset_handle);

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

    let mut quest_status = QuestStatus::default();
    // quest_status.record_node_done(&demo_node_id);

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
            EnteringNode(demo_node_id_clone),
            PlayedCards::default(),
            IsReadyToGo(false),
            Deck::new()
                .with_card(hack)
                .with_card(hack)
                .with_card(hack)
                .with_card(hack)
                .with_card(hack)
                .with_card(hack)
                .with_card(hack)
                .with_card(hack)
                .with_card(hack)
                .with_card(hack)
                .with_card(hack)
                .with_card(hack)
                .with_card(hack)
                .with_card(hack)
                .with_card(card_0)
                .with_card(card_1)
                .with_card(card_2)
                .with_card(card_3)
                .with_card(card_4)
                .with_card(card_5)
                .with_card(card_fiddle)
                .with_card(card_bb),
        ))
        .id();

    // World map things

    let board = commands
        .spawn((Board("Network Map".into()),))
        .with_children(|board| {
            board.spawn((
                NFNode,
                ForNode(demo_node_id.clone()),
                TerminalRendering::default(),
                BoardPosition(UVec2 { x: 0, y: 0 }),
                BoardPiece("Demo Node".to_owned()),
                BoardSize(UVec2 { x: 4, y: 1 }),
                Name::new("Board piece 1"),
            ));
            board.spawn((
                NFNode,
                ForNode(NodeId::new("node:demo", 1)),
                RequiredNodes(vec![demo_node_id.clone()]),
                TerminalRendering::default(),
                BoardPiece("Next Demo Node".to_owned()),
                BoardPosition(UVec2 { x: 6, y: 4 }),
                BoardSize(UVec2 { x: 4, y: 1 }),
                Name::new("Board piece 2"),
            ));
            board.spawn((
                NFNode,
                RequiredNodes(vec![demo_node_id.clone()]),
                TerminalRendering::default(),
                BoardPiece("Shop Node".to_owned()),
                BoardPosition(UVec2 { x: 0, y: 4 }),
                BoardSize(UVec2 { x: 4, y: 1 }),
                Name::new("Board piece 2"),
            ));
            board.spawn((
                NFNode,
                ForNode(NodeId::new("node:demo", 2)),
                RequiredNodes(vec![NodeId::new("node:demo", 1)]),
                TerminalRendering::default(),
                BoardPiece("Demo Node".to_owned()),
                BoardPosition(UVec2 { x: 14, y: 4 }),
                BoardSize(UVec2 { x: 4, y: 1 }),
                Name::new("Board piece 2"),
            ));
        })
        .id();

    let board_ui_root = commands
        .spawn((
            Name::new("Network map"),
            TerminalRendering::new(Vec::new()),
            CalculatedSizeTty(UVec2 { x: 400, y: 500 }),
            StyleTty(taffy::style::Style {
                flex_direction: taffy::style::FlexDirection::Column,
                ..default()
            }),
            LayoutRoot,
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
                    content_pane.spawn((
                        StyleTty(taffy::prelude::Style {
                            size: Size {
                                width: Dimension::Points(14.),
                                height: Dimension::Auto,
                            },
                            flex_direction: FlexDirection::Column,
                            ..default()
                        }),
                        Name::new("Menu Bar"),
                        TerminalRendering::new(vec!["WIP".to_owned()]),
                    ));
                    content_pane.spawn((
                        Name::new("Demo map background"),
                        ForPlayer(player),
                        BoardUi(board),
                        BoardBackground(asset_server.load("nightfall/demo_map.charmi.toml")),
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
    res_demo_state.node_id = None;
    res_demo_state.player_id = Some(player);

    log::debug!("Demo startup executed");
}
