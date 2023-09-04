use game_core::card::{
    Action, ActionEffect, ActionRange, Actions, Card, Deck, Description, MaximumSize,
    MovementSpeed, Prereqs, Prerequisite, RangeShape,
};
use game_core::node::{
    AccessPoint, AccessPointLoadingRule, ActiveCurio, Curio, CurrentTurn, InNode, IsReadyToGo,
    IsTapped, Mon, MovesTaken, Node, NodeOp, NodePiece, OnTeam, Pickup, PlayedCards, Team,
    TeamColor, TeamPhase, Teams,
};
use game_core::op::OpResult;
use game_core::player::PlayerBundle;
use game_core::prelude::*;

use crate::fx::Fx;
use crate::input_event::KeyCode;
use crate::layout::LayoutEvent;
use crate::node_ui::{NodeCursor, NodeUiOp, ShowNode};
use crate::prelude::KeyEvent;
use crate::KeyMap;

/// Plugin to set up temporary entities and systems while I get the game set up
pub struct DemoPlugin;

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
    asset_server: Res<AssetServer>,
    fx: Res<Fx>,
    mut ev_keys: EventReader<KeyEvent>,
    nodes: Query<(Entity, &EntityGrid, Option<&NodeCursor>), With<Node>>,
    mut layout_events: EventReader<LayoutEvent>,
) {
    for layout_event in layout_events.iter() {
        log::trace!("LAYOUT EVENT: {:?}", layout_event);
    }
    for KeyEvent { code, .. } in ev_keys.iter() {
        if *code == KeyCode::Char('/') {
            log::debug!("Debug event occured");
            log::debug!(
                "Pickup sound load state: {:?}",
                asset_server.get_load_state(fx.pickup_sound.clone())
            );
            log::debug!(
                "Animation load state: {:?}",
                asset_server.get_load_state(fx.charmia.clone())
            );

            for (_, entity_grid, cursor) in nodes.iter() {
                log::debug!("# Node ({:?})", cursor);
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

fn demo_startup(mut commands: Commands, mut load_node_writer: EventWriter<ShowNode>) {
    let player_team = commands
        .spawn((Team, TeamColor::Blue, TeamPhase::Setup))
        .id();
    let enemy_team = commands.spawn((Team, TeamColor::Red, TeamPhase::Play)).id();
    let act_slice = commands
        .spawn((
            Action {
                name: "Slice".to_owned(),
            },
            ActionRange::new(1),
            ActionEffect::Damage(2),
            Description::new("Deletes 2 sectors from target"),
        ))
        .id();
    let act_stone = commands
        .spawn((
            Action {
                name: "Stone".to_owned(),
            },
            ActionRange::new(2),
            ActionEffect::Damage(1),
            Description::new("(Range 2) Deletes 1 sectors from target"),
        ))
        .id();
    let act_glitch = commands
        .spawn((
            Action {
                name: "Glitch".to_owned(),
            },
            ActionRange::new(1),
            ActionEffect::Damage(2),
            Description::new("Deletes 2 sectors from target"),
        ))
        .id();
    let act_dice = commands
        .spawn((
            Action {
                name: "Dice".to_owned(),
            },
            ActionRange::new(2),
            ActionEffect::Damage(3),
            Description::new("(Req Size: 3) Deletes 3 sectors from target"),
            Prereqs(vec![Prerequisite::MinSize(3)]),
        ))
        .id();
    let act_thrice = commands
        .spawn((
            Action {
                name: "Thrice".to_owned(),
            },
            ActionRange::new(4),
            ActionEffect::Damage(3),
            Description::new("Testare says HELLO"),
        ))
        .id();
    let act_ping = commands
        .spawn(((
            Action {
                name: "Ping".to_owned(),
            },
            ActionRange::new(2),
            ActionEffect::Damage(2),
            Description::new("Range(2) Deletes 2 sectors from target"),
        ),))
        .id();
    let act_square = commands
        .spawn(((
            Action {
                name: "Square".to_owned(),
            },
            ActionRange::new(2).shaped(RangeShape::Square),
            ActionEffect::Damage(2),
            Description::new("Range(2[]) Deletes 2 sectors from target"),
        ),))
        .id();
    let act_calamari = commands
        .spawn(((
            Action {
                name: "Calamari".to_owned(),
            },
            ActionRange::new(1).headless(true),
            ActionEffect::Damage(2),
            Description::new("Range(1*) Deletes 2 sectors from target"),
        ),))
        .id();
    let act_circle = commands
        .spawn(((
            Action {
                name: "Circle".to_owned(),
            },
            ActionRange::new(5).shaped(RangeShape::Circle),
            ActionEffect::Damage(2),
            Description::new("Range(2o) Deletes 2 sectors from target"),
        ),))
        .id();
    let act_ff_bow = commands
        .spawn(((
            Action {
                name: "FF Bow".to_owned(),
            },
            ActionRange::new(4).min_range(3),
            ActionEffect::Damage(2),
            Description::new("Range(2-3) Deletes 2 sectors from target"),
        ),))
        .id();

    let hack = commands
        .spawn((
            Card::new("Hack", "curio:hack", None),
            MaximumSize(4),
            MovementSpeed(3),
            Actions(vec![act_slice, act_dice]),
            Description::new("Basic attack program"),
        ))
        .id();
    let card_0 = commands
        .spawn((
            Card::new("Sling", "curio:sling", None),
            Description::new("Basic attack program"),
            MaximumSize(3),
            MovementSpeed(2),
            Actions(vec![act_stone]),
        ))
        .id();
    let card_1 = commands
        .spawn((
            Card::new("Bit Man", "curio:bit_man", None),
            Description::new("Makes sectors of the grid appear or disappear"),
        ))
        .id();
    let card_2 = commands
        .spawn((
            Card::new("Bug", "curio:bug", None),
            Description::new("Fast, cheap, and out of control"),
            MaximumSize(1),
            MovementSpeed(5),
            Actions(vec![act_glitch]),
        ))
        .id();
    let card_3 = commands
        .spawn((
            Card::new("Super Bug", "curio:death", None),
            Description::new("Testing utility"),
            MaximumSize(4),
            MovementSpeed(6),
            Actions(vec![act_square, act_calamari, act_circle, act_ff_bow]),
        ))
        .id();
    let card_4 = commands
        .spawn((
            Card::new("Card4", "curio:hack", None),
            Description::new("Basic attack program4"),
        ))
        .id();
    let card_5 = commands
        .spawn((
            Card::new(
                "Data Doctor Pro",
                "curio:data_doctor_pro",
                Some("DataDocPro"),
            ),
            Description::new("He's gonna get you"),
            MovementSpeed(5),
            MaximumSize(8),
        ))
        .id();
    let node = commands
        .spawn((
            Node,
            Teams(vec![player_team, enemy_team]),
            CurrentTurn(player_team),
            AccessPointLoadingRule::Staggered,
            ActiveCurio::default(),
            EntityGrid::from_shape_string("EwALACCAAz7447/vP/7x+AABPh7/+O/7jz/4gAMIAA==").unwrap(),
            Name::new("Demo Node"),
        ))
        .with_children(|node| {
            let node_id = node.parent_entity();

            /*
            node.spawn((
                NodePiece::new("curio:hack"),
                OnTeam(player_team),
                Curio::new("Hack"),
                Actions(vec![act_slice, act_dice, act_thrice]),
                Description::new("Basic attack program"),
                MaximumSize(4),
                MovementSpeed(5),
                MovesTaken(1),
                IsTapped(false),
            ))
            .add_to_grid(node_id, vec![(5, 4), (5, 3)]);
            */

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
                NodePiece::new("curio:death"),
                OnTeam(enemy_team),
                MovementSpeed(2),
                IsTapped(true),
                Actions(vec![act_ping]),
            ))
            .add_to_grid(node_id, vec![(2, 5)]);
            node.spawn((
                NodePiece::new("curio:death"),
                OnTeam(enemy_team),
                MovementSpeed(2),
                IsTapped(false),
                Actions(vec![act_ping]),
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
                .with_card(card_5),
        ))
        .id();

    load_node_writer.send(ShowNode { node, player });
    log::debug!("Demo startup executed");
}
