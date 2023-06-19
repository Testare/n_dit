use crossterm::event::{KeyCode, KeyEvent};
use game_core::card::{
    Action, ActionEffect, Actions, Card, Deck, Description, MaximumSize, MovementSpeed,
};
use game_core::node::{
    AccessPoint, AccessPointLoadingRule, ActiveCurio, Curio, CurrentTurn, InNode, IsReadyToGo,
    IsTapped, MovesTaken, Node, NodeOp, NodePiece, OnTeam, Pickup, PlayedCards, Team, TeamColor,
    TeamPhase, Teams,
};
use game_core::player::PlayerBundle;
use game_core::prelude::*;

use crate::term::layout::LayoutEvent;
use crate::term::node_ui::{NodeCursor, ShowNode};
use crate::term::KeyMap;

/// Plugin to set up temporary entities and systems while I get the game set up
pub struct DemoPlugin;

impl Plugin for DemoPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(demo_startup)
            .add_system(debug_key)
            .add_system(log_ops);
    }
}

fn demo_startup(mut commands: Commands, mut load_node_writer: EventWriter<ShowNode>) {
    let player_team = commands
        .spawn((Team, TeamColor::Blue, TeamPhase::Setup))
        .id();
    let enemy_team = commands.spawn((Team, TeamColor::Red, TeamPhase::Play)).id();
    let hack = commands
        .spawn((
            Card::new("Hack", "curio:hack", None),
            MaximumSize(4),
            MovementSpeed(2),
            Actions::new(vec![Action {
                name: "Slice".to_owned(),
                range: 1,
                effect: ActionEffect::Damage(2),
                description: "Deletes 2 sectors from target".to_owned(),
                prereqs: default(),
            }]),
            Description::new("Basic attack program"),
        ))
        .id();
    let card_0 = commands
        .spawn((
            Card::new("Sling", "curio:sling", None),
            Description::new("Basic attack program"),
            MaximumSize(3),
            MovementSpeed(2),
            Actions::new(vec![Action {
                name: "Shoot".to_owned(),
                range: 2,
                effect: ActionEffect::Damage(1),
                description: "(Range 2) Deletes 1 sectors from target".to_owned(),
                prereqs: default(),
            }]),
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
            Actions::new(vec![Action {
                name: "Glitch".to_owned(),
                range: 1,
                effect: ActionEffect::Damage(2),
                description: "Deletes 2 sectors from target".to_owned(),
                prereqs: default(),
            }]),
        ))
        .id();
    let card_3 = commands
        .spawn((
            Card::new("Card3", "curio:hack", None),
            Description::new("Basic attack program3"),
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
        ))
        .with_children(|node| {
            let node_id = node.parent_entity();

            node.spawn((
                NodePiece::new("curio:hack"),
                OnTeam(player_team),
                Curio::new("Hack"),
                Actions::new(vec![
                    Action {
                        name: "Slice".to_owned(),
                        range: 1,
                        effect: ActionEffect::Damage(2),
                        description: "Deletes 2 sectors from target".to_owned(),
                        prereqs: default(),
                    },
                    Action {
                        name: "Dice".to_owned(),
                        range: 1,
                        effect: ActionEffect::Damage(3),
                        description: "Deletes 3 sectors from target".to_owned(),
                        prereqs: default(),
                    },
                ]),
                Description::new("Basic attack program"),
                MaximumSize(4),
                MovementSpeed(3),
                MovesTaken(1),
                IsTapped(false),
            ))
            .add_to_grid(node_id, vec![(5, 4), (5, 3)]);

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
                Pickup::Card(hack),
                NodePiece::new("pickup:card"),
                Description::new("A card! Get this card! /it;s a good card! A very good card!"),
            ))
            .add_to_grid(node_id, vec![(4, 3)]);

            node.spawn((
                NodePiece::new("curio:death"),
                OnTeam(enemy_team),
                MovementSpeed(2),
                IsTapped(true),
                Actions::new(vec![Action {
                    name: "Ping".to_owned(),
                    range: 2,
                    effect: ActionEffect::Damage(2),
                    description: "Range(2) Deletes 2 sectors from target".to_owned(),
                    prereqs: default(),
                }]),
            ))
            .add_to_grid(node_id, vec![(2, 5)]);
            node.spawn((
                NodePiece::new("curio:death"),
                OnTeam(enemy_team),
                MovementSpeed(2),
                IsTapped(false),
                Actions::new(vec![Action {
                    name: "Ping".to_owned(),
                    range: 2,
                    effect: ActionEffect::Damage(2),
                    description: "Range(2) Deletes 2 sectors from target".to_owned(),
                    prereqs: default(),
                }]),
            ))
            .add_to_grid(node_id, vec![(12, 3)]);
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

fn log_ops(mut node_ops: EventReader<Op<NodeOp>>) {
    for op in node_ops.iter() {
        log::debug!("NODE_OP {:?}", op)
    }
}

fn debug_key(
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

            for (_, entity_grid, cursor) in nodes.iter() {
                log::debug!("# Node ({:?})", cursor);
                for entry in entity_grid.entities() {
                    log::debug!("Entity: {:?}", entry);
                }
            }
        }
    }
}
