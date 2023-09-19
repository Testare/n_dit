use game_core::card::{
    ActionDefinition, Actions, Card, CardDefinition, Deck, Description, MaximumSize, MovementSpeed,
};
use game_core::node::{
    AccessPoint, AccessPointLoadingRule, ActiveCurio, AiThread, Curio, CurrentTurn, InNode,
    IsReadyToGo, IsTapped, Mon, MovesTaken, NoOpAction, Node, NodeBattleIntelligence, NodeOp,
    NodePiece, OnTeam, Pickup, PlayedCards, SimpleAiCurioOrder, Team, TeamColor, TeamPhase,
    TeamStatus, Teams, VictoryStatus,
};
use game_core::op::OpResult;
use game_core::player::PlayerBundle;
use game_core::prelude::*;
use game_core::registry::Reg;

use crate::fx::Fx;
use crate::input_event::KeyCode;
use crate::layout::LayoutEvent;
use crate::node_ui::{NodeCursor, NodeUiOp, ShowNode, NodeGlyph};
use crate::prelude::KeyEvent;
use crate::KeyMap;

/// Plugin to set up temporary entities and systems while I get the game set up
pub struct DemoPlugin;

#[derive(Component)]
pub struct DebugEntityMarker;

#[derive(Component)]
pub struct CardAssetPointer {
    handle: Handle<CardDefinition>,
}

impl Plugin for DemoPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, demo_startup)
            .add_systems(PostUpdate, (debug_key, log_ops, log_op_results));
    }
}

fn log_ops(mut ops_node: EventReader<Op<NodeOp>>, mut ops_node_ui: EventReader<Op<NodeUiOp>>) {
    for op in ops_node.iter() {
        log::debug!("NODE_OP {:?}", op)
    }
    for op in ops_node_ui.iter() {
        log::debug!("NODE_UI_OP {:?}", op)
    }
}

fn log_op_results(mut node_ops: EventReader<OpResult<NodeOp>>) {
    for op in node_ops.iter() {
        log::debug!("NODE_OP_RESULT {:?}", op)
    }
}

fn debug_key(
    reg_curio_display: Res<Reg<NodeGlyph>>,
    asset_server: Res<AssetServer>,
    card_assets: Res<Assets<CardDefinition>>,
    action_assets: Res<Assets<ActionDefinition>>,
    fx: Res<Fx>,
    mut ev_keys: EventReader<KeyEvent>,
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
    debug_asset: Query<&CardAssetPointer>,
    mut layout_events: EventReader<LayoutEvent>,
) {
    for layout_event in layout_events.iter() {
        log::trace!("LAYOUT EVENT: {:?}", layout_event);
    }
    for KeyEvent { code, .. } in ev_keys.iter() {
        if *code == KeyCode::Char('/') {
            log::debug!("Node Glyph Reg: {:?}", reg_curio_display);
            for CardAssetPointer { handle } in debug_asset.iter() {
                log::debug!(
                    "ASSET LOAD STATE: {:?} ",
                    asset_server.get_load_state(handle)
                );
                let card_asset = card_assets.get(handle);
                log::debug!("CARD ASSET: {:?}", card_asset);
                if let Some(card_asset) = card_asset {
                    for action_handle in card_asset.actions().iter() {
                        log::debug!("CARD ACTION: {:?}", action_assets.get(action_handle))
                    }
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
        }
    }
}

fn demo_startup(
    asset_server: Res<AssetServer>,
    no_op: Res<NoOpAction>,
    mut commands: Commands,
    mut load_node_writer: EventWriter<ShowNode>,
) {
    let _card_asset = commands.spawn((
        DebugEntityMarker,
        CardAssetPointer {
            handle: asset_server.load("nightfall/lvl1.cards.json#Hack"),
        },
    ));
    let player_team = commands
        .spawn((Team, TeamColor::Blue, TeamPhase::Setup))
        .id();
    let enemy_team = commands.spawn((Team, TeamColor::Red, TeamPhase::Play)).id();
    let act_phaser = asset_server.load("nightfall/enemies.actions.json#Phaser");
    let hack = commands
        .spawn((Card::new(
            "Hack",
            "curio:hack",
            None,
            asset_server.load("/nightfall/lvl1.cards.json#Hack"),
        ),))
        .id();
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
                "Card4",
                "curio:hack",
                None,
                asset_server.load("nightfall/lvl1.cards.json#Hack"),
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
    let node = commands
        .spawn((
            Node,
            Teams(vec![player_team, enemy_team]),
            CurrentTurn(player_team),
            AccessPointLoadingRule::Staggered,
            TeamStatus(
                [
                    (player_team, VictoryStatus::Undecided),
                    (enemy_team, VictoryStatus::Undecided),
                ]
                .into_iter()
                .collect(),
            ),
            ActiveCurio::default(),
            EntityGrid::from_shape_string("EwALACCAAz7447/vP/7x+AABPh7/+O/7jz/4gAMIAA==").unwrap(),
            Name::new("Demo Node"),
        ))
        .with_children(|node| {
            let node_id = node.parent_entity();
            node.spawn((
                NodePiece::new("env:access_point"),
                AccessPoint::default(),
                OnTeam(player_team),
            ))
            .add_to_grid(node_id, vec![(6, 2)]);
            node.spawn((
                NodePiece::new("env:access_point"),
                AccessPoint::default(),
                OnTeam(player_team),
            ))
            .add_to_grid(node_id, vec![(12, 2)]);
            node.spawn((
                NodePiece::new("env:access_point"),
                AccessPoint::default(),
                OnTeam(player_team),
            ))
            .add_to_grid(node_id, vec![(12, 10)]);
            node.spawn((
                Pickup::Card(hack),
                NodePiece::new("pickup:card"),
                Description::new("A card! Get this card! It's a good card! A very good card!"),
            ))
            .add_to_grid(node_id, vec![(4, 3)]);
            node.spawn((
                Pickup::Mon(Mon(1000)),
                NodePiece::new("pickup:mon"),
                Description::new("Put food on the table, and cards in your deck"),
            ))
            .add_to_grid(node_id, vec![(11, 10)]);

            node.spawn((
                Actions(vec![act_phaser.clone(), (**no_op).clone()]),
                Curio::new("Shinigami"),
                IsTapped(false),
                MaximumSize(7),
                MovementSpeed(2),
                MovesTaken(0),
                NodePiece::new("Attack Dog"),
                SimpleAiCurioOrder(1),
                OnTeam(enemy_team),
            ))
            .add_to_grid(node_id, vec![(2, 5)]);
            node.spawn((
                Actions(vec![act_phaser.clone(), (**no_op).clone()]),
                Curio::new("Shinigami"),
                IsTapped(false),
                MaximumSize(7),
                MovementSpeed(2),
                MovesTaken(0),
                NodePiece::new("Attack Dog"),
                SimpleAiCurioOrder(0),
                OnTeam(enemy_team),
            ))
            .add_to_grid(
                node_id,
                vec![
                    (12, 3),
                    (13, 3),
                    (14, 3),
                    (15, 3),
                    (15, 4),
                    (15, 5),
                    (16, 5),
                ],
            );
        })
        .id();
    commands.spawn((
        PlayerBundle::default(),
        IsReadyToGo(true),
        InNode(node),
        OnTeam(enemy_team),
        NodeBattleIntelligence::Simple,
        Name::new("Jackson"),
        AiThread::default(),
    ));
    let player = commands
        .spawn((
            PlayerBundle::default(),
            KeyMap::default(),
            OnTeam(player_team),
            InNode(node),
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

    load_node_writer.send(ShowNode { node, player });
    log::debug!("Demo startup executed");
}
