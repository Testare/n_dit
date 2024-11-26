use bevy::ecs::system::{EntityCommand, EntityCommands};

use super::EntityGrid;
use crate::prelude::*;

pub trait AddToGrid {
    fn add_to_grid<P: Into<UVec2>>(&mut self, grid: Entity, points: Vec<P>) -> &mut Self;
}

#[derive(Debug)]
pub struct AddToGridCommand {
    grid_entity: Entity,
    points: Vec<UVec2>,
}

impl<'a> AddToGrid for EntityCommands<'a> {
    fn add_to_grid<P: Into<UVec2>>(&mut self, grid_entity: Entity, points: Vec<P>) -> &mut Self {
        if points.is_empty() {
            panic!("cannot add to grid when there are no points");
        }
        let command = AddToGridCommand {
            grid_entity,
            points: points.into_iter().map(|p| p.into()).collect(),
        };
        self.add(command);
        self
    }
}

impl EntityCommand for AddToGridCommand {
    fn apply(self, id: Entity, world: &mut World) {
        let AddToGridCommand {
            grid_entity,
            points,
        } = self;
        if let Some(mut map) = world.entity_mut(grid_entity).get_mut::<EntityGrid>() {
            // TODO push item, then push_back any extra points
            let mut pts_iter = points.iter();
            if let Some(head) = pts_iter.next() {
                if let Some(item_key) = map.put_item(*head, id) {
                    // TODO modify grid_map not to need item keys
                    for pt in pts_iter {
                        map.push_back(*pt, item_key);
                    }
                }
            } else {
                let grid_name = world
                    .entity(self.grid_entity)
                    .get::<Name>()
                    .map(|name| name.as_str())
                    .unwrap_or("unnamed");
                let my_name = world
                    .entity(id)
                    .get::<Name>()
                    .map(|name| name.as_str())
                    .unwrap_or("unnamed");
                log::error!(
                    "{}[{:?}] cannot add [{}]{:?} to EntityGrid since it does not have any points",
                    grid_name,
                    self.grid_entity,
                    my_name,
                    id
                );
            }
        } else {
            let grid_name = world
                .entity(self.grid_entity)
                .get::<Name>()
                .map(|name| name.as_str())
                .unwrap_or("unnamed");
            let my_name = world
                .entity(id)
                .get::<Name>()
                .map(|name| name.as_str())
                .unwrap_or("unnamed");
            log::error!(
                "{}[{:?}] does not have an EntityGrid for [{}]{:?} to be added to",
                grid_name,
                self.grid_entity,
                my_name,
                id
            );
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
