use bevy::prelude::UVec3;

use crate::prelude::*;

pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        
    }
}

#[derive(Component, Default, Reflect)]
#[reflect(Component)]
pub struct Map(pub String);

#[derive(Component, Default, Reflect)]
#[reflect(Component)]
pub struct MapPosition(pub UVec2);

#[derive(Component, Default, Reflect)]
#[reflect(Component)]
pub struct InteractionPoint;

#[derive(Event)]
struct Interact;