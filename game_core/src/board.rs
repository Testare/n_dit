use crate::prelude::*;

pub struct BoardPlugin;

impl Plugin for BoardPlugin {
    fn build(&self, _app: &mut App) {}
}

#[derive(Component, Default, Reflect)]
#[reflect(Component)]
pub struct Board(pub String);

#[derive(Component, Default, Reflect)]
#[reflect(Component)]
pub struct BoardPosition(pub UVec2);

#[derive(Component, Default, Reflect)]
#[reflect(Component)]
pub enum BoardShape {
    #[default]
    Free,
    SimpleRect(UVec2),
}

#[derive(Component, Default, Reflect)]
#[reflect(Component)]
pub struct InteractionPoint;

#[derive(Event)]
struct Interact;
