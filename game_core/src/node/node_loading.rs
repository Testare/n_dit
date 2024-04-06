use std::ops::Deref;

use bevy::ecs::query::Has;

use super::{
    Curio, CurioFromCard, EnteringNode, InNode, IsReadyToGo, IsTapped, MovesTaken, Node, NodePiece,
    OnTeam, Teams,
};
use crate::card::{Actions, CardDefinition, Description, MaximumSize, MovementSpeed};
use crate::player::Player;
use crate::prelude::*;
use crate::registry::{Reg, Registry};

pub struct NodeLoadingPlugin;

impl Plugin for NodeLoadingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PostUpdate, (sys_enter_node_when_ready, sys_load_curios))
            .add_plugins(Reg::<NodeScene>::default())
            .register_type::<CurioFromCard>();
    }
}

#[derive(Debug)]
pub struct NodeScene;

impl Registry for NodeScene {
    const REGISTRY_NAME: &'static str = "core:node_scenes";
    type Value = String;
}

impl CurioFromCard {
    fn get_handle(&mut self, asset_server: &AssetServer) -> Handle<CardDefinition> {
        if let CurioFromCard::Path(card_name) = self.deref() {
            *self = CurioFromCard::Handle(asset_server.load(card_name));
        }

        match self {
            CurioFromCard::Handle(handle) => handle.clone(),
            CurioFromCard::Path(_) => {
                unreachable!("LoadCurioFromCard should be transmuted to Handle by this point")
            },
        }
    }
}

impl Default for CurioFromCard {
    fn default() -> Self {
        CurioFromCard::Path(String::new())
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
            nodes.iter().find(|node_q| node_q.0 .0 == *node_id)
        {
            // TODO check that all curios are loaded first
            if node_is_ready {
                commands.entity(player_id).remove::<EnteringNode>().insert((
                    InNode(node_entity),
                    OnTeam(teams[0]),
                    IsReadyToGo(false),
                ));
            }
        }
    }
}

fn sys_load_curios(
    mut commands: Commands,
    assets: Res<Assets<CardDefinition>>,
    asset_server: Res<AssetServer>,
    mut curios_from_card: Query<(Entity, &mut CurioFromCard)>,
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
                .remove::<CurioFromCard>();
        }
    }
}
