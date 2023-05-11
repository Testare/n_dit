use game_core::{EntityGrid, Mon, Node, NodePiece, Team};
use game_core::prelude::*;
use crossterm::event::{Event as CrosstermEvent, KeyCode, KeyEvent};

/// Plugin to set up temporary entities and systems while I get the game set up
pub struct DemoPlugin;

impl Plugin for DemoPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(demo_startup).add_system(debug_key);
    }
}

fn demo_startup(mut commands: Commands) {
    commands
        .spawn((
            Node, 
            EntityGrid::from_shape_string("EwALACCAAz7447/vP/7x+AABPh7/+O/7jz/4gAMIAA==").unwrap()
        ))
        .with_children(|node| {
            let node_id = node.parent_entity();

            node
                .spawn((
                    Mon(500), 
                    NodePiece::new("curio:hack"),
                    Team::Enemy
                ))
                .add_to_grid(node_id, vec![(4, 4), (4, 3)]);
            node
                .spawn((Mon(700), NodePiece::new("pickup:card")))
                .add_to_grid(node_id, vec![(3, 3)]);

        });

    // commands.get_entity(node).unwrap().insert(entity_grid);
    log::debug!("Demo startup executed");
}

// TODO figure out how to add entity to GridMap well, custom Command?

fn debug_key(mut inputs: EventReader<CrosstermEvent>, nodes: Query<&EntityGrid, With<Node>>) {
    for input in inputs.iter() {
        if let CrosstermEvent::Key(KeyEvent {
            code: KeyCode::Char('d'),
            ..
        }) = input
        {
            log::debug!("Debug event occured");
            for entity_grid in nodes.iter() {
                log::debug!("# Node");
                for entry in entity_grid.entities() {
                    log::debug!("Entity: {:?}", entry);
                }
            }
        }
    }
}
