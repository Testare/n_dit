use bevy::reflect::TypePath;

use super::{CurioQ, NodeOp};
use crate::card::{Action, MaximumSize, MovementSpeed};
use crate::node::{
    key, ActiveCurio, Curio, CurrentTurn, InNode, NoOpAction, Node, OnTeam, Pickup, Team,
    TeamPhase, TeamStatus, Teams, VictoryStatus,
};
use crate::opv2::{OpError, OpErrorUtils, OpImplResult, OpRegistrar, OpV2};
use crate::player::Player;
use crate::prelude::*;

impl OpV2 for NodeOp {
    fn register_systems(mut registrar: OpRegistrar<Self>)
    where
        Self: Sized + TypePath + FromReflect,
    {
        registrar
            .register_op(opsys_node_movement)
            .register_op(opsys_node_action)
            .register_op(opsys_node_activate)
            .register_op(opsys_node_access_point)
            .register_op(opsys_node_ready)
            .register_op(opsys_node_end_turn);
    }

    fn system_index(&self) -> usize {
        match self {
            Self::MoveActiveCurio { .. } => 0,
            Self::PerformCurioAction { .. } => 1,
            Self::ActivateCurio { .. } => 2,
            Self::LoadAccessPoint { .. } => 3,
            Self::UnloadAccessPoint { .. } => 3,
            Self::ReadyToGo { .. } => 4,
            Self::EndTurn => 5,
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
    mut curios: Query<CurioQ, With<Curio>>,
    pickups: Query<&Pickup>,
) -> OpImplResult {
    if let NodeOp::MoveActiveCurio { dir } = node_op {
        let mut metadata = Metadata::default();
        let (player_team_id, node_id) = players.get(player).critical()?;
        let (mut grid, current_turn, mut active_curio) = nodes.get_mut(node_id).critical()?;

        if player_team_id != current_turn {
            Err("Not this player's turn".invalid())?;
        }
        if *team_phases.get(player_team_id).critical()? == TeamPhase::Setup {
            Err("Can't move pieces during setup phase".invalid())?;
        }
        let active_curio_id = active_curio.ok_or("No active curio to move")?;

        metadata.put(key::NODE_ID, node_id).critical()?;
        metadata.put(key::CURIO, active_curio_id).critical()?;
        let mut curio_q = curios.get_mut(active_curio_id).critical()?;
        debug_assert!(!**curio_q.tapped, "a tapped curio was active");
        let movement_speed = **curio_q.movement_speed.ok_or("Movement speed is 0")?;
        if movement_speed == **curio_q.moves_taken {
            return Err("No movement remains")?;
        }
        let head = grid
            .head(active_curio_id)
            .ok_or("Active curio not in grid".critical())?;
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
        if grid.len_of(active_curio_id) as u32 > curio_q.max_size.map(|ms| **ms).unwrap_or(1) {
            metadata
                .put(
                    key::DROPPED_SQUARE,
                    grid.back(active_curio_id)
                        .expect("piece should be at least one square long"),
                )
                .critical()?;
            grid.pop_back(active_curio_id);
        }
        let remaining_moves = movement_speed - **curio_q.moves_taken;

        metadata
            .put(key::REMAINING_MOVES, remaining_moves)
            .critical()?;
        if movement_speed == **curio_q.moves_taken
            && curio_q
                .actions
                .map(|curio_actions| {
                    !curio_actions
                        .iter()
                        .any(|action| action.id() != no_op_action.id())
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

fn opsys_node_action(
    In((player, node_op)): In<(Entity, NodeOp)>,
    ast_action: Res<Assets<Action>>,
    mut nodes: Query<
        (
            &mut EntityGrid,
            AsDerefCopied<CurrentTurn>,
            AsDerefMut<ActiveCurio>,
            &Teams,
            AsDerefMut<TeamStatus>,
        ),
        With<Node>,
    >,
    players: Query<(AsDerefCopied<OnTeam>, AsDerefCopied<InNode>), With<Player>>,
    team_phases: Query<&TeamPhase, With<Team>>,
    mut curios: ParamSet<(
        Query<CurioQ, With<Curio>>,
        Query<(AsDerefMut<MaximumSize>, AsDerefMut<MovementSpeed>), With<Curio>>,
    )>,
    curio_teams: Query<AsDerefCopied<OnTeam>, With<Curio>>,
) -> OpImplResult {
    if let NodeOp::PerformCurioAction {
        action_id,
        curio,
        target,
    } = node_op
    {
        let (player_team_id, node_id) = players.get(player).critical()?;
        let (mut grid, current_turn, mut active_curio, teams, mut team_status) =
            nodes.get_mut(node_id).critical()?;

        if player_team_id != current_turn {
            Err("Not this player's turn".invalid())?;
        }
        if *team_phases.get(player_team_id).critical()? == TeamPhase::Setup {
            Err("Can't perform actions during setup phase".invalid())?;
        }
        if active_curio.is_some() && curio.is_some() && *active_curio != curio {
            Err("There's already an active curio and it's not that one".invalid())?;
        }
        let curio_id = active_curio
            .or(curio)
            .ok_or("No curio to perform that action".invalid())?;

        let curios_p0 = curios.p0();
        let curio_q = get_assert!(curio_id, curios_p0)
            .ok_or("Calling curio action on entity that is not a curio".critical())?;
        if **curio_q.tapped {
            Err("Curio is tapped".invalid())?;
        }
        let action_def = curio_q
            .actions
            .ok_or("Curio has no actions".invalid())?
            .iter()
            .find_map(|action_handle| {
                let action_def = ast_action.get(action_handle)?;
                (action_def.id() == action_id).then_some(action_def)
            })
            .ok_or("That action is not defined".invalid())?;

        if let Some(range) = action_def.range() {
            if !range.in_range_of(grid.as_ref(), curio_id, target) {
                Err("Target out of range".invalid())?;
            }
        }
        for prereq in action_def.prereqs() {
            if !prereq.satisfied(&grid, curio_id, target) {
                Err("Prerequisites not satisfied".invalid())?;
            }
        }
        if !action_def
            .target()
            .valid_target(&grid, curio_id, target, |id| curio_teams.get(id).ok())
        {
            Err("Invalid target".invalid())?;
        }
        let mut metadata = Metadata::new();
        let effect_metadata = action_def
            .effects()
            .iter()
            .map(|effect| effect.apply_effect(&mut grid, curio_id, target, &mut curios.p1()))
            .collect::<Result<Vec<_>, _>>()
            .critical()?;

        metadata
            .put_optional(key::EFFECTS, Metadata::aggregate(effect_metadata))
            .critical()?;

        let self_effects = action_def
            .self_effects()
            .iter()
            .filter_map(|effect| {
                let head = grid.head(curio_id)?;
                Some(effect.apply_effect(&mut grid, curio_id, head, &mut curios.p1()))
            })
            .collect::<Result<Vec<_>, _>>()
            .critical()?;
        metadata
            .put_optional(key::SELF_EFFECTS, Metadata::aggregate(self_effects))
            .critical()?;
        metadata.put(key::NODE_ID, node_id).critical()?;

        // Have to drop curios_p0 temporarily to apply actions, then we need to bring them back to tap the piece
        let mut curios_p0 = curios.p0();
        let mut curio_q = get_assert_mut!(curio_id, curios_p0)
            .ok_or("Curio disappeared mid operation".critical())?;
        **curio_q.tapped = true;
        *active_curio = None;

        // WIP Test victory conditions
        // Need to rework for different victory conditions, such as obtaining key items, or time limits
        for team in teams.iter() {
            if team_status[team].is_undecided() {
                let still_in_this = curios
                    .p0()
                    .iter()
                    .any(|curio_q| **curio_q.team == *team && grid.contains_key(curio_q.id));
                if !still_in_this {
                    team_status.insert(*team, VictoryStatus::Loss);
                }
            }
        }
        let remaining_teams: Vec<Entity> = teams
            .iter()
            .filter(|team| team_status[*team].is_undecided())
            .copied()
            .collect();
        if remaining_teams.len() == 1 {
            let team = remaining_teams[0];
            let is_victory_flawed = curios.p0().iter().any(|curio_q| {
                curio_q.in_node == node_id
                    && **curio_q.team == team
                    && !grid.contains_key(curio_q.id)
            });
            let victory_status = if is_victory_flawed {
                VictoryStatus::Victory
            } else {
                VictoryStatus::PerfectVictory
            };
            team_status.insert(team, victory_status);
        }
        Ok(metadata)
    } else {
        Err(OpError::MismatchedOpSystem)
    }
}

fn opsys_node_activate(
    In((player, node_op)): In<(Entity, NodeOp)>,
    mut nodes: Query<(AsDerefCopied<CurrentTurn>, AsDerefMut<ActiveCurio>), With<Node>>,
    players: Query<(AsDerefCopied<OnTeam>, AsDerefCopied<InNode>), With<Player>>,
    team_phases: Query<&TeamPhase, With<Team>>,
    mut curios: Query<CurioQ, With<Curio>>,
) -> OpImplResult {
    if let NodeOp::ActivateCurio { curio_id } = node_op {
        let mut metadata = Metadata::default();
        let (player_team_id, node_id) = players.get(player).critical()?;
        let (current_turn, mut active_curio) = nodes.get_mut(node_id).critical()?;

        if player_team_id != current_turn {
            Err("Not this player's turn".invalid())?;
        }
        if *team_phases.get(player_team_id).critical()? == TeamPhase::Setup {
            Err("Can't activate pieces during setup phase".invalid())?;
        }

        let target_curio = curios
            .get(curio_id)
            .map_err(|_| "Target curio not found".invalid())?;
        if **target_curio.team != player_team_id {
            Err("Cannot activate pieces on the other team".invalid())?;
        }
        if **target_curio.tapped {
            Err("Cannot activate tapped curio".invalid())?
        }
        if let Some(last_active) = *active_curio {
            if last_active == curio_id {
                Err("That curio is already active".invalid())?;
            }
            metadata
                .put(key::DEACTIVATED_CURIO, last_active)
                .critical()?; // Recoverable?
            **curios.get_mut(last_active).critical()?.tapped = true;
        }
        *active_curio = Some(curio_id);
        Ok(metadata)
    } else {
        Err(OpError::MismatchedOpSystem)
    }
}

fn opsys_node_access_point(In((player, node_op)): In<(Entity, NodeOp)>) -> OpImplResult {
    if let NodeOp::LoadAccessPoint {
        access_point_id, ..
    }
    | NodeOp::UnloadAccessPoint { access_point_id } = node_op
    {}
    // Load AND unload
    Ok(default())
}

fn opsys_node_ready(In((player, node_op)): In<(Entity, NodeOp)>) -> OpImplResult {
    Ok(default())
}

fn opsys_node_end_turn(In((player, node_op)): In<(Entity, NodeOp)>) -> OpImplResult {
    Ok(default())
}
