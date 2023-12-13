use std::ops::Deref;

use bevy::ecs::query::Has;

use super::{Curio, EnteringNode, InNode, IsTapped, MovesTaken, Node, NodePiece, OnTeam, Teams};
use crate::card::{Actions, CardDefinition, Description, MaximumSize, MovementSpeed};
use crate::player::Player;
use crate::prelude::*;

pub struct NodeLoadingPlugin;

impl Plugin for NodeLoadingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PostUpdate, (sys_enter_node_when_ready, sys_load_curios))
            .register_type::<LoadCurioFromCard>();
    }
}

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub enum LoadCurioFromCard {
    Path(String),
    Handle(Handle<CardDefinition>),
}

impl LoadCurioFromCard {
    fn get_handle(&mut self, asset_server: &AssetServer) -> Handle<CardDefinition> {
        if let LoadCurioFromCard::Path(card_name) = self.deref() {
            *self = LoadCurioFromCard::Handle(asset_server.load(card_name));
        }

        match self {
            LoadCurioFromCard::Handle(handle) => handle.clone(),
            LoadCurioFromCard::Path(_) => {
                unreachable!("LoadCurioFromCard should be transmuted to Handle by this point")
            },
        }
    }
}

impl Default for LoadCurioFromCard {
    fn default() -> Self {
        LoadCurioFromCard::Path(String::new())
    }
}

fn sys_enter_node_when_ready(
    mut commands: Commands,
    players_entering: Query<(Entity, AsDeref<EnteringNode>), With<Player>>,
    nodes: Query<(&Node, Entity, AsDeref<Teams>, Has<EntityGrid>)>,
) {
    // Note: Node loading kickoff should either happen here or in an op
    for (player_id, node_id) in players_entering.iter() {
        if let Some((_, node_entity, teams, node_is_ready)) =
            nodes.iter().find(|(node, _, _, _)| node.0 == *node_id)
        {
            // TODO check that all curios are loaded first
            if node_is_ready {
                commands
                    .entity(player_id)
                    .remove::<EnteringNode>()
                    .insert((InNode(node_entity), OnTeam(teams[0])));
            }
        }
    }
}

fn sys_load_curios(
    mut commands: Commands,
    assets: Res<Assets<CardDefinition>>,
    asset_server: Res<AssetServer>,
    mut curios_from_card: Query<(Entity, &mut LoadCurioFromCard)>,
) {
    for (id, mut lcfc) in curios_from_card.iter_mut() {
        let handle = lcfc.get_handle(&asset_server);
        if let Some(card_def) = assets.get(handle) {
            commands
                .entity(id)
                .insert((
                    Actions(card_def.actions().clone()),
                    Curio::new(card_def.id()),
                    NodePiece::new(card_def.id()),
                    Description::new(card_def.description().to_owned()),
                    IsTapped::default(),
                    MaximumSize(card_def.max_size()),
                    MovementSpeed(card_def.movement_speed()),
                    MovesTaken::default(),
                ))
                .remove::<LoadCurioFromCard>();
        }
    }
}
