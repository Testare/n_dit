use crate::prelude::*;

/// Marker for a player entity.
#[derive(Component, Debug)]
pub struct Player;

#[derive(Clone, Component, Copy, Debug)]
pub struct ForPlayer(pub Entity);

#[derive(Bundle)]
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


#[derive(Component, Debug, FromReflect, Reflect)]
pub struct Players(Vec<Entity>);
