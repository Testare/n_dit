use super::prelude::*;
use bevy::ecs::system::{EntityCommand, EntityCommands};
use old_game_core::GridMap;

type Point = UVec2;

#[derive(Component, Deref, DerefMut)]
struct EntityMap  {
    grid: GridMap<Entity>
}

trait AddToGrid {
    fn add_to_grid(&mut self, grid: Entity, points: Vec<Point>) -> &mut Self;
}

struct AddToGridCommand {
    grid_entity: Entity,
    points: Vec<Point>,
}

impl <'w, 's, 'a> AddToGrid for EntityCommands<'w, 's, 'a> {
    fn add_to_grid(&mut self, grid_entity: Entity, points: Vec<Point>) -> &mut Self {
        if points.len() < 1 {
            panic!("cannot add to grid when there are no points");
        }
        let command = AddToGridCommand {
            grid_entity,
            points,
        };
        self.add(command);
        self
    }
}

impl EntityCommand for AddToGridCommand {
    fn write(self, id: Entity, world: &mut World) {
        let AddToGridCommand { grid_entity, points } = self;
        if let Some(mut map) = world.entity_mut(grid_entity).get_mut::<EntityMap>() {
            // TODO push item, then push_back any extra points
            let mut pts_iter = points.iter();
            if let Some(head) = pts_iter.next() {
                if let Some(item_key) = map.grid.put_item((head.x as usize, head.y as usize), id) {
                    // TODO modify grid_map not to need item keys
                    for pt in pts_iter {
                        map.grid.push_back((pt.x as usize, pt.y as usize), item_key);
                    }
                }
            } else {
                let grid_name = world.entity(self.grid_entity).get::<Name>().map(|name|name.as_str()).unwrap_or("unnamed");
                let my_name = world.entity(id).get::<Name>().map(|name|name.as_str()).unwrap_or("unnamed");
                log::error!("{}[{:?}] cannot add [{}]{:?} to EntityMap since it does not have any points", grid_name, self.grid_entity, my_name, id);
            }

        } else {
            let grid_name = world.entity(self.grid_entity).get::<Name>().map(|name|name.as_str()).unwrap_or("unnamed");
            let my_name = world.entity(id).get::<Name>().map(|name|name.as_str()).unwrap_or("unnamed");
            log::error!("{}[{:?}] does not have an EntityMap for [{}]{:?} to be added to", grid_name, self.grid_entity, my_name, id);
        }
    }
}

/*
 * So what is the expected form factor for adding entities to a grid?
 *
 * I suppose 
 *
 * let node = commands.spawn(..).id();
 *
 * let mon = commands.spawn(..).add_to_grid(node).id();
 *
 * node.add_child(mon)
 *
 * 
 *
 * */
