use bevy::ecs::query::Has;

use super::{EnteringNode, InNode, Node, OnTeam, Teams};
use crate::player::Player;
use crate::prelude::*;

pub struct NodeLoadingPlugin;

impl Plugin for NodeLoadingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, sys_enter_node_when_ready);
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
            if node_is_ready {
                // TODO add player to the appropriate team
                commands
                    .entity(player_id)
                    .remove::<EnteringNode>()
                    .insert((InNode(node_entity), OnTeam(teams[0])));
            }
        }
    }
}
