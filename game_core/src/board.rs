use crate::prelude::*;

#[derive(Debug, Default)]
pub struct BoardPlugin;

impl Plugin for BoardPlugin {
    fn build(&self, _app: &mut App) {}
}

#[derive(Component, Debug, Default, Deref, Reflect)]
#[reflect(Component)]
pub struct Board(pub String);

// Simple string displaying basic information on the piece
// Might need to rethink this one for dynamic piece information
#[derive(Component, Debug, Default, Deref, Reflect)]
#[reflect(Component)]
pub struct SimplePieceInfo(pub String);

#[derive(Component, Debug, Default, Deref, Reflect)]
#[reflect(Component)]
pub struct BoardPiece(pub String);

#[derive(Clone, Component, Copy, Debug, Default, Deref, DerefMut, Reflect)]
#[reflect(Component)]
pub struct BoardPosition(pub Vec2);

#[derive(Clone, Component, Copy, Debug, Deref, DerefMut, Reflect)]
#[reflect(Component)]
pub struct BoardSize(pub Vec2);

impl Default for BoardSize {
    fn default() -> Self {
        BoardSize(Vec2 { x: 1.0, y: 1.0 })
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
