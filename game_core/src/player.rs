use crate::prelude::*;

/// Marker for a player entity.
#[derive(Component, Debug)]
pub struct Player;

#[derive(Clone, Component, Copy, Debug, Deref)]
pub struct ForPlayer(pub Entity);

#[derive(Clone, Component, Debug)]
pub struct ForMultiPlayer(pub Vec<Entity>);

#[derive(Bundle, Debug)]
pub struct PlayerBundle {
    player_marker: Player,
}

impl Default for PlayerBundle {
    fn default() -> Self {
        PlayerBundle {
            player_marker: Player,
        }
    }
}

#[derive(Component, Debug, Reflect)]
pub struct Players(Vec<Entity>);
