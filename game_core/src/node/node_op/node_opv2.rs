use bevy::reflect::TypePath;

use crate::card::{Action, MaximumSize, MovementSpeed};
use crate::node::{NoOpAction, CurrentTurn, ActiveCurio, Teams, TeamStatus, OnTeam, InNode, Node, TeamPhase, Team, Curio, Pickup, key};
use crate::opv2::{OpV2, PrimeOpQueue, OpSysResult, OpRegistrar, OpError, OpResult, OpErrorUtils};
use crate::player::Player;
use crate::prelude::*;

use super::{NodeOp, NodeOpError, CurioQ};

impl OpV2 for NodeOp {
    type Queue = PrimeOpQueue;
    fn register_systems(mut registrar: OpRegistrar<Self>)
        where
            Self: Sized + TypePath + FromReflect {
                registrar.register_op(opsys_node_movement);
        
    }
    fn system_index(&self) -> usize {
        match self {
            Self::MoveActiveCurio { .. } => 0,
            _ => 1,
        }
    }
}

fn opsys_node_movement(
    In((player, node_op)): In<(Entity, NodeOp)>,
    no_op_action: Res<NoOpAction>,
    mut nodes: Query<
        (
            &mut EntityGrid,
            AsDerefCopied<CurrentTurn>,
            AsDerefMut<ActiveCurio>,
        ),
        With<Node>,
    >,
    players: Query<(AsDerefCopied<OnTeam>, AsDerefCopied<InNode>), With<Player>>,
    team_phases: Query<&TeamPhase, With<Team>>,
    mut curios: ParamSet<(
        Query<CurioQ, With<Curio>>,
        Query<(AsDerefMut<MaximumSize>, AsDerefMut<MovementSpeed>), With<Curio>>,
    )>,
    pickups: Query<&Pickup>,
) -> OpSysResult {
    if let NodeOp::MoveActiveCurio { dir } = node_op {
        let mut metadata = Metadata::default();
        let (player_team_id, node_id) = players.get(player).critical()?;
        let (mut grid, current_turn, mut active_curio) = nodes.get_mut(node_id).critical()?;

        if player_team_id != current_turn {
            "Not this player's turn".invalid()?;
        }
        if *team_phases.get(player_team_id).critical()? == TeamPhase::Setup {
            "Can't move pieces during setup phase".invalid()?;
        }
        let active_curio_id = active_curio.ok_or("No active curio to move")?;

        metadata.put(key::NODE_ID, node_id).critical()?;
        metadata.put(key::CURIO, active_curio_id).critical()?;
        let mut curios = curios.p0();
        let mut curio_q = curios
            .get_mut(active_curio_id)
            .critical()?;
        debug_assert!(!**curio_q.tapped, "a tapped curio was active");
        let movement_speed =
            **curio_q.movement_speed.ok_or("Movement speed is 0")?;
        if movement_speed == **curio_q.moves_taken {
            return Err("No movement remains")?;
        }
        let head = grid.head(active_curio_id)
            .map_or_else(||"Active curio not in grid".critical(), Ok)?;
        let next_pt = head + dir;
        metadata.put(key::TARGET_POINT, next_pt).critical()?;
        if grid.square_is_closed(next_pt) {
            return Err("Cannot move into closed square")?;
        }
        if let Some(entity_at_pt) = grid.item_at(next_pt) {
            if entity_at_pt == active_curio_id {
                // Curios can move onto their own squares
            } else if let Ok(pickup) = pickups.get(entity_at_pt) {
                grid.remove_entity(entity_at_pt);
                metadata.put(key::PICKUP, pickup).critical()?;
                log::debug!("Picked up: {:?}", pickup);
            } else {
                return Err("Invalid target")?;
            }
        }
        grid.push_front(next_pt, active_curio_id);
        **curio_q.moves_taken += 1;
        if grid.len_of(active_curio_id) as u32
            > curio_q.max_size.map(|ms| **ms).unwrap_or(1)
        {
            metadata.put(
                key::DROPPED_SQUARE,
                grid.back(active_curio_id)
                    .expect("piece should be at least one square long"),
            ).critical()?;
            grid.pop_back(active_curio_id);
        }
        let remaining_moves = movement_speed - **curio_q.moves_taken;

        metadata.put(key::REMAINING_MOVES, remaining_moves).critical()?;
        if movement_speed == **curio_q.moves_taken
            && curio_q
                .actions
                .as_ref()
                .map(|curio_actions| {
                    curio_actions.iter().any(|action| *action != **no_op_action)
                })
                .unwrap_or(true)
        {
            metadata.put(key::TAPPED, true).critical()?;
            **curio_q.tapped = true;
            *active_curio = None;
        }
        Ok(metadata)
    } else {
        Err(OpError::MismatchedOpSystem)
    }
}