use super::game_core::{EntityGrid, Mon, Node as NDNode, NodePiece};
use bevy::{prelude::*, app::ScheduleRunnerSettings};
use crossterm::event::{Event as CrosstermEvent, KeyCode, KeyEvent};

/// Plugin to set up temporary entities and systems while I get the game set up
pub struct DemoPlugin;

impl Plugin for DemoPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(demo_startup).add_system(debug_key);
    }
}

fn demo_startup(mut commands: Commands) {
    let mut entity_grid =
        EntityGrid::new_from_shape("EwALACCAAz7447/vP/7x+AABPh7/+O/7jz/4gAMIAA==").unwrap();

    let mon_entity_1 = commands.spawn((Mon(500), NodePiece::new("mon"))).id();
    entity_grid.put_item((4, 4), mon_entity_1);

    let node = commands
        .spawn((NDNode, entity_grid))
        .add_child(mon_entity_1)
        .id();
    // commands.get_entity(node).unwrap().insert(entity_grid);
    log::debug!("Demo startup executed");
}

// TODO figure out how to add entity to GridMap well, custom Command?

fn debug_key(mut inputs: EventReader<CrosstermEvent>, nodes: Query<&EntityGrid, With<NDNode>>) {
    for input in inputs.iter() {
        if let CrosstermEvent::Key(KeyEvent {
            code: KeyCode::Char('d'),
            ..
        }) = input
        {
            log::debug!("Debug event occured");
            for entity_grid in nodes.iter() {
                log::debug!("# Node");
                for entry in entity_grid.entries() {
                    log::debug!("Entity: {:?}", entry);
                }
            }
        }
    }
}
