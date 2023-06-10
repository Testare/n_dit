use crate::prelude::*;

/// Alias for usize. A number to indicate the number associated with a player.
/// Not called "Player ID" to avoid confusion with Entity
type PN = usize;

/// Marker for a player entity.
#[derive(Component, Debug)]
pub struct Player(PN);

/// A Marker for a specific player entity. This is so that systems/resources/components specific to
/// a certain player can quickly find the player entity with generics.
///
#[derive(Component, Debug)]
pub struct PlayerN<const P: PN>();

#[derive(Component, Debug)]
pub struct ForPlayerN<const P: PN>;

impl Player {
    fn pn(&self) -> PN {
        self.0
    }
}

#[derive(Bundle)]
pub struct PlayerBundle<const P: PN> {
    pn_marker: PlayerN<P>,
    player_marker: Player,
}

impl<const P: PN> Default for PlayerBundle<P> {
    fn default() -> Self {
        PlayerBundle {
            pn_marker: PlayerN::<P>(),
            player_marker: Player(P),
        }
    }
}
