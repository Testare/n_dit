use crate::prelude::*;

#[derive(Debug, Default)]
pub struct BoardPlugin;

impl Plugin for BoardPlugin {
    fn build(&self, _app: &mut App) {}
}

#[derive(Component, Debug, Default, Reflect)]
#[reflect(Component)]
pub struct Board(pub String);

#[derive(Component, Debug, Default, Reflect)]
#[reflect(Component)]
pub struct BoardPiece(pub String);

#[derive(Clone, Component, Copy, Debug, Default, Deref, DerefMut, Reflect)]
#[reflect(Component)]
pub struct BoardPosition(pub UVec2);

#[derive(Clone, Component, Copy, Debug, Deref, DerefMut, Reflect)]
#[reflect(Component)]
pub struct BoardSize(pub UVec2);

impl Default for BoardSize {
    fn default() -> Self {
        BoardSize(UVec2 { x: 1, y: 1 })
    }
}

#[derive(Component, Debug, Default, Reflect)]
#[reflect(Component)]
pub enum BoardShape {
    #[default]
    Free,
    SimpleRect(UVec2),
}

#[derive(Component, Debug, Default, Reflect)]
#[reflect(Component)]
pub struct InteractionPoint;

#[derive(Event)]
struct Interact;
