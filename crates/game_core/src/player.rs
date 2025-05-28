use bevy::ecs::query::{QueryData, QueryFilter};
use bevy::ecs::system::QueryLens;

use crate::prelude::*;

#[derive(Debug)]
pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<ForPlayer>().register_type::<Player>();
    }
}

#[derive(Clone, Component, Copy, Debug, Deref, Reflect)]
pub struct ForPlayer(#[entities] pub Entity);

impl ForPlayer {
    /// Until entity relations comes up with a better API, this will help
    /// ### Panics
    /// Panics if Q does not have &ForPlayer
    pub fn get<'a, Q: QueryData, F: QueryFilter>(
        query: &'a mut Query<Q, F>,
        player_id: Entity,
    ) -> Option<<<Q as QueryData>::ReadOnly as QueryData>::Item<'a>> {
        let mut lens: QueryLens<&ForPlayer> = query.transmute_lens();
        let index = lens
            .query()
            .iter()
            .position(|for_player| for_player.0 == player_id)?;
        Some(query.iter().nth(index).expect(
            "lens was created out of this query, should have the same data in the same places",
        ))
    }

    /// Until entity relations comes up with a better API, this will help
    /// ### Panics
    /// Panics if Q does not have &ForPlayer
    pub fn get_mut<'a, Q: QueryData, F: QueryFilter>(
        query: &'a mut Query<Q, F>,
        player_id: Entity,
    ) -> Option<<Q as QueryData>::Item<'a>> {
        let mut lens: QueryLens<&ForPlayer> = query.transmute_lens();
        let index = lens
            .query()
            .iter()
            .position(|for_player| for_player.0 == player_id)?;
        Some(query.iter_mut().nth(index).expect(
            "lens was created out of this query, should have the same data in the same places",
        ))
    }
}

/// Marker for a player entity.
#[derive(Component, Clone, Copy, Debug, Default, Reflect)]
#[reflect(Component)]
pub struct Player;

/// Marker for "Non-Computer Player"
#[derive(Component, Clone, Copy, Debug, Default, Reflect)]
#[reflect(Component)]
pub struct Ncp;

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
