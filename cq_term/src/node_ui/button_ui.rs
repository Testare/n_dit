use bevy::ecs::query::Has;
use game_core::node::{
    AccessPoint, AccessPointLoadingRule, IsReadyToGo, Node, NodeOp, NodePiece, OnTeam, Teams,
};
use game_core::op::OpResult;
use game_core::player::{ForPlayer, Player};

use crate::layout::LayoutMouseTargetDisabled;
use crate::prelude::*;

#[derive(Clone, Copy, Component, Reflect)]
pub struct ReadyButton;

pub fn sys_ready_button_disable(
    mut commands: Commands,
    mut ev_node_op_result: EventReader<OpResult<NodeOp>>,
    ready_buttons: IndexedQuery<
        ForPlayer,
        (Entity, Has<LayoutMouseTargetDisabled>),
        With<ReadyButton>,
    >,
    nodes: Query<(&AccessPointLoadingRule, &EntityGrid, &Teams), With<Node>>,
    players: Query<(Option<&IsReadyToGo>,), With<Player>>,
    access_points: Query<(&ForPlayer, Entity, &OnTeam, &AccessPoint), With<NodePiece>>,
) {
    // TODO make this a run condition?
    // for OpResult { .. }
    for node_op_result in ev_node_op_result.iter() {
        if let OpResult {
            result: Ok(_),
            source: Op { player, op },
        } = node_op_result
        {
            let should_be_enabled = match op {
                NodeOp::LoadAccessPoint { .. } => true,
                NodeOp::ReadyToGo => false,
                NodeOp::UnloadAccessPoint { .. } => {
                    // TODO check if other accesss points are still loaded
                    // TODO actually emit this nodeopresult
                    false
                },
                _ => continue,
            };
            if let Ok((id, button_is_disabled)) = ready_buttons.get_for(*player) {
                if button_is_disabled && should_be_enabled {
                    commands.entity(id).remove::<LayoutMouseTargetDisabled>();
                } else if !button_is_disabled && !should_be_enabled {
                    commands.entity(id).insert(LayoutMouseTargetDisabled);
                }
            }
        }
    }
}
