use super::NodeOp;
use crate::node::{OnTeam, Team};
use crate::op::OpResult;
use crate::player::Player;
use crate::prelude::*;
use crate::NDitCoreSet;

#[derive(Debug, Default)]
pub struct NodeOpUndoPlugin {
    default_allowed_undo_depth: UndoDepth,
}

impl Plugin for NodeOpUndoPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(self.default_allowed_undo_depth)
            .add_systems(
                Update,
                (
                    sys_add_undo_queue,
                    sys_record_node_ops.in_set(NDitCoreSet::PostProcessCommands),
                ),
            );
    }
}

#[derive(Clone, Copy, Component, Debug, Default, Resource, Reflect)]
#[reflect(Component)]
pub enum UndoDepth {
    OnlyMovement,
    #[default]
    ActionAndMovement,
    WholeTurn,
}

#[derive(Component, Debug, Default, Deref, DerefMut)]
pub struct NodeUndoStack(Vec<OpResult<NodeOp>>);

pub fn sys_add_undo_queue(
    mut commands: Commands,
    q_team: Query<Entity, (With<Team>, Without<NodeUndoStack>)>,
) {
    for id in q_team.iter() {
        commands.entity(id).insert(NodeUndoStack::default());
    }
}

pub fn sys_record_node_ops(
    mut evr_node_op: EventReader<OpResult<NodeOp>>,
    q_player: Query<&OnTeam, With<Player>>,
    mut q_team: Query<(DebugName, &mut NodeUndoStack), With<Team>>,
) {
    for op_result in evr_node_op.read() {
        (|| {
            // try
            let &OnTeam(team_id) = q_player.get(op_result.source()).ok()?;
            let (_, mut undo_queue) = q_team.get_mut(team_id).ok()?;
            match op_result.op() {
                NodeOp::ActivateCurio { .. } => {
                    undo_queue.0.clear();
                    undo_queue.0.push(op_result.clone());
                },
                NodeOp::MoveActiveCurio { .. } | NodeOp::PerformCurioAction { .. } => {
                    undo_queue.0.push(op_result.clone())
                },
                NodeOp::EnterNode(_) => {},
                _ => {
                    undo_queue.0.clear();
                },
            }

            log::debug!("Undo queue size: {}", undo_queue.0.len());

            Some(())
        })();
    }
}
