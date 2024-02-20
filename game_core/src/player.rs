use bevy::ecs::entity::{EntityMapper, MapEntities};

use crate::prelude::*;

#[derive(Debug)]
pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<ForPlayer>().register_type::<Player>();
    }
}

#[derive(Clone, Component, Copy, Debug, Deref, Reflect)]
pub struct ForPlayer(pub Entity);

impl MapEntities for ForPlayer {
    fn map_entities<M: EntityMapper>(&mut self, entity_mapper: &mut M) {
        self.0 = entity_mapper.map_entity(self.0);
    }
}

/// Marker for a player entity.
#[derive(Component, Clone, Copy, Debug, Default, Reflect)]
#[reflect(Component)]
pub struct Player;

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
