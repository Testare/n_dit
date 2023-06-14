use crate::prelude::*;

/// Marker for a player entity.
#[derive(Component, Debug)]
pub struct Player(usize);

/// A Marker for a specific player entity. This is so that systems/resources/components specific to
/// a certain player can quickly find the player entity with generics.
///
#[derive(Component, Debug)]
pub struct PlayerN<const P: usize>();

#[derive(Component, Debug)]
pub struct ForPlayerN(pub usize);

impl Player {
    fn pn(&self) -> usize {
        self.0
    }
}

#[derive(Bundle)]
pub struct PlayerBundle<const P: usize> {
    pn_marker: PlayerN<P>,
    player_marker: Player,
}

impl<const P: usize> Default for PlayerBundle<P> {
    fn default() -> Self {
        PlayerBundle {
            pn_marker: PlayerN::<P>(),
            player_marker: Player(P),
        }
    }
}


#[derive(Component, Debug, FromReflect, Reflect)]
pub struct Players(Vec<Entity>);
