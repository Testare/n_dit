use crossterm::event::{Event as CrosstermEvent, KeyCode, KeyEvent};
use game_core::card::{Card, Deck};
use game_core::player::PlayerBundle;
use game_core::prelude::*;
use game_core::{
    AccessPoint, Action, Actions, Curio, Description, EntityGrid, IsTapped, MaximumSize,
    MovementSpeed, MovesTaken, Node, NodePiece, Pickup, Team,
};

use crate::term::node_ui::{NodeCursor, ShowNode};

/// Plugin to set up temporary entities and systems while I get the game set up
pub struct DemoPlugin;

impl Plugin for DemoPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(demo_startup).add_system(debug_key);
    }
}

fn demo_startup(mut commands: Commands, mut load_node_writer: EventWriter<ShowNode>) {
    let example_card = commands
        .spawn((Card::new("Hack"), Description::new("Basic attack program")))
        .id();
    commands.spawn((
        PlayerBundle::<0>::default(),
        Deck::new().with_card(example_card).with_card(example_card),
    ));
    let node = commands
        .spawn((
            Node,
            EntityGrid::from_shape_string("EwALACCAAz7447/vP/7x+AABPh7/+O/7jz/4gAMIAA==").unwrap(),
        ))
        .with_children(|node| {
            let node_id = node.parent_entity();

            node.spawn((
                NodePiece::new("curio:hack"),
                Team::Player,
                Curio::new("Hack"),
                Actions::new(vec![Action {
                    name: "Slice".to_owned(),
                    range: 1,
                    description: "Deletes 2 sectors from target".to_owned(),
                }]),
                Description::new("Basic attack program"),
                MaximumSize(4),
                MovementSpeed(3),
                MovesTaken(1),
                IsTapped(false),
            ))
            .add_to_grid(node_id, vec![(5, 4), (5, 3)]);

            node.spawn((NodePiece::new("env:access_point"), AccessPoint::default()))
                .add_to_grid(node_id, vec![(6, 2)]);
            node.spawn((
                Pickup::Card(example_card),
                NodePiece::new("pickup:card"),
                Description::new("A card! Get this card! /it;s a good card! A very good card!"),
            ))
            .add_to_grid(node_id, vec![(4, 3)]);

            node.spawn((
                NodePiece::new("curio:death"),
                Team::Enemy,
                MovementSpeed(2),
                IsTapped(true),
            ))
            .add_to_grid(node_id, vec![(2, 5)]);
            node.spawn((
                NodePiece::new("curio:death"),
                Team::Enemy,
                MovementSpeed(2),
                IsTapped(false),
            ))
            .add_to_grid(node_id, vec![(12, 3)]);
        })
        .id();

    load_node_writer.send(ShowNode(node));
    log::debug!("Demo startup executed");
}

fn debug_key(
    mut inputs: EventReader<CrosstermEvent>,
    nodes: Query<(Entity, &EntityGrid, Option<&NodeCursor>), With<Node>>,
) {
    for input in inputs.iter() {
        if let CrosstermEvent::Key(KeyEvent {
            code: KeyCode::Char('d'),
            ..
        }) = input
        {
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
