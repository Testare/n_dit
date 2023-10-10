use crate::prelude::*;

pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, _app: &mut App) {}
}

#[derive(Component, Default, Reflect)]
#[reflect(Component)]
pub struct Map(pub String);

#[derive(Component, Default, Reflect)]
#[reflect(Component)]
pub struct MapPosition(pub UVec2);

#[derive(Component, Default, Reflect)]
#[reflect(Component)]
pub enum MapShape {
    #[default]
    Free,
    SimpleRect(UVec2),
}

#[derive(Component, Default, Reflect)]
#[reflect(Component)]
pub struct InteractionPoint;

#[derive(Event)]
struct Interact;
