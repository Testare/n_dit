pub mod node_op_undo;

use std::borrow::Cow;

use bevy::ecs::query::QueryData;
use bevy::hierarchy::DespawnRecursiveExt;
use bevy::reflect::TypePath;
use bevy::scene::DynamicScene;

use self::daddy::Daddy;
use self::node_op_undo::NodeUndoStack;
use super::{Claimed, EnteringNode, NodeId, NodeScene};
use crate::card::{
    Action, ActionEffect, Actions, CardQuery, Deck, Description, MaximumSize, MovementSpeed,
    NO_OP_ACTION_ID,
};
use crate::configuration::PlayerConfiguration;
use crate::node::{
    key, AccessPoint, AccessPointLoadingRule, ActiveCurio, Curio, CurrentTurn, InNode, IsReadyToGo,
    IsTapped, MovesTaken, NoOpAction, Node, NodePiece, OnTeam, Pickup, PlayedCards, Team,
    TeamPhase, TeamStatus, Teams, VictoryStatus,
};
use crate::op::{CoreOps, Op, OpError, OpErrorUtils, OpImplResult, OpRegistrar};
use crate::player::{Ncp, Player};
use crate::prelude::*;
use crate::quest::QuestStatus;
use crate::registry::Reg;

#[derive(Clone, Debug, Reflect)]
pub enum NodeOp {
    PerformCurioAction {
        action_id: Cow<'static, str>,
        curio: Option<Entity>,
        target: UVec2,
    },
    MoveActiveCurio {
        dir: Compass,
    },
    ActivateCurio {
        curio_id: Entity,
    },
    LoadAccessPoint {
        access_point_id: Entity,
        card_id: Entity,
    },
    UnloadAccessPoint {
        access_point_id: Entity,
    },
    ReadyToGo,
    EndTurn,
    TelegraphAction {
        action_id: Cow<'static, str>,
    },
    EnterNode(NodeId),
    QuitNode(NodeId),
    Undo,
}

#[derive(Debug, QueryData)]
#[query_data(mutable)]
pub struct CurioQ {
    id: Entity,
    in_node: AsDerefCopied<Parent>,
    team: &'static OnTeam,
    tapped: &'static mut IsTapped,
    moves_taken: &'static mut MovesTaken,
    movement_speed: Option<&'static mut MovementSpeed>,
    max_size: Option<&'static mut MaximumSize>,
    actions: Option<&'static Actions>,
}

const ACCESS_POINT_DISPLAY_ID: &str = "env:access_point";

impl Op for NodeOp {
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
            .register_op(opsys_node_end_turn)
            .register_op(opsys_telegraph_action)
            .register_op(opsys_node_enter_battle)
            .register_op(opsys_node_quit_battle)
            .register_op(opsys_node_undo);
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
            Self::TelegraphAction { .. } => 6,
            Self::EnterNode(_) => 7,
            Self::QuitNode(_) => 8,
            Self::Undo => 9,
        }
    }
}

fn opsys_node_movement(
    In((player, node_op)): In<(Entity, NodeOp)>,
    mut commands: Commands,
    mut res_core_ops: ResMut<CoreOps>,
    res_no_op_action: Res<NoOpAction>,
    mut nodes: Query<
        (
            &mut EntityGrid,
            AsDerefCopied<CurrentTurn>,
            AsDerefCopied<ActiveCurio>,
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
        let (mut grid, current_turn, active_curio) = nodes.get_mut(node_id).critical()?;

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
                metadata.put(key::PICKUP_ID, entity_at_pt).critical()?;
                commands
                    .entity(entity_at_pt)
                    .insert(Claimed { player, node_id });
                log::debug!("Picked up: {pickup:?} ({entity_at_pt:?}) at {next_pt:?}");
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
                        .any(|action| action.id() != res_no_op_action.id())
                })
                .unwrap_or(true)
        {
            res_core_ops.request(
                player,
                NodeOp::PerformCurioAction {
                    action_id: NO_OP_ACTION_ID,
                    curio: None,
                    target: UVec2::default(),
                },
            );
        }
        Ok(metadata)
    } else {
        Err(OpError::MismatchedOpSystem)
    }
}

fn opsys_node_action(
    In((player, node_op)): In<(Entity, NodeOp)>,
    ast_action: Res<Assets<Action>>,
    mut res_core_ops: ResMut<CoreOps>,
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
    players: Query<
        (
            AsDerefCopied<OnTeam>,
            AsDerefCopied<InNode>,
            Option<&PlayerConfiguration>,
        ),
        With<Player>,
    >,
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
        let (player_team_id, node_id, player_config) = players.get(player).critical()?;
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
        metadata.put(key::CURIO, curio_id).critical()?;
        if let Some(last_active_id) = *active_curio {
            if last_active_id != curio_id {
                metadata.put(key::SKIPPED_ACTIVATION, true).critical()?;
                metadata
                    .put(key::DEACTIVATED_CURIO, last_active_id)
                    .critical()?; // Recoverable?
            } else {
                metadata.put(key::SKIPPED_ACTIVATION, false).critical()?;
            }
        } else {
            metadata.put(key::SKIPPED_ACTIVATION, true).critical()?;
        }
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

        // TODO probably don't bother if the game is over
        if player_config
            .and_then(|config| config.node.as_ref())
            .map(|node_config| node_config.end_turn_after_all_pieces_tap)
            .unwrap_or(false)
        {
            let all_curios_tapped = curios
                .p0()
                .iter()
                .all(|curio_q| **curio_q.team != player_team_id || **curio_q.tapped);
            if all_curios_tapped {
                res_core_ops.request(player, NodeOp::EndTurn);
            }
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

fn opsys_node_access_point(
    In((player, node_op)): In<(Entity, NodeOp)>,
    mut commands: Commands,
    cards: Query<CardQuery>,
    mut access_points: Query<(&mut AccessPoint, &mut NodePiece)>,
    mut players: Query<(&mut PlayedCards, &Deck, &InNode), With<Player>>,
) -> OpImplResult {
    let (access_point_id, next_card_id) = match node_op {
        NodeOp::LoadAccessPoint {
            access_point_id,
            card_id,
        } => (access_point_id, Some(card_id)),
        NodeOp::UnloadAccessPoint { access_point_id } => (access_point_id, None),
        _ => return Err(OpError::MismatchedOpSystem),
    };
    let mut metadata = Metadata::new();
    let (mut access_point, mut node_piece) = access_points
        .get_mut(access_point_id)
        .map_err(|_| "No such access point".invalid())?;
    let (mut played_cards, deck, &InNode(node_id)) = players.get_mut(player).critical()?;
    metadata.put(key::NODE_ID, node_id).invalid()?;

    if access_point.card.is_none() && next_card_id.is_none() {
        Err("Access point is already unloaded".invalid())?;
    }
    if access_point.card == next_card_id {
        Err("That is already loaded".invalid())?;
    }

    let mut access_point_commands = commands.entity(access_point_id);
    if let Some(next_card_id) = next_card_id {
        if !played_cards.can_be_played(deck, next_card_id) {
            Err("Already played all of those".invalid())?;
        }
        let card_q = cards
            .get(next_card_id)
            .map_err(|_| "Cannot find that card or it is not loaded".invalid())?;
        metadata.put(key::CARD, next_card_id).critical()?;

        // VALIDATIONS COMPLETE, COMMENCE MUTATING STATE
        if let Some(old_card_id) = access_point.card {
            let withdrawn_successfully = played_cards.withdraw_card_from(old_card_id, node_id);
            if !withdrawn_successfully {
                Err("Attempting to unload card that wasn't played here".critical())?;
            }
        }

        played_cards.play_card_to(deck, next_card_id, node_id);

        node_piece.set_display_id(card_q.base_name.clone());

        access_point_commands.insert((
            Description::new(card_q.description.to_owned()),
            MovementSpeed(card_q.movement_speed),
            MaximumSize(card_q.max_size),
            Actions(card_q.actions.clone()),
        ));
    } else {
        // Unloading card
        if let Some(old_card_id) = access_point.card {
            let withdrawn_successfully = played_cards.withdraw_card_from(old_card_id, node_id);
            if !withdrawn_successfully {
                Err("Attempting to unload card that wasn't played here".critical())?;
            }
        }

        node_piece.set_display_id(ACCESS_POINT_DISPLAY_ID.to_owned());
        access_point_commands.remove::<(Description, MovementSpeed, MaximumSize, Actions)>();
    };
    access_point.card = next_card_id;

    Ok(metadata)
}

fn opsys_node_ready(
    In((player, node_op)): In<(Entity, NodeOp)>,
    no_op_action: Res<NoOpAction>,
    mut commands: Commands,
    cards: Query<CardQuery>,
    mut players: Query<(Entity, &OnTeam, &InNode, AsDerefMut<IsReadyToGo>), With<Player>>,
    mut team_phases: Query<&mut TeamPhase, With<Team>>,
    access_points: Query<(Entity, &OnTeam, &AccessPoint), With<NodePiece>>,
    mut nodes: Query<(&AccessPointLoadingRule, &mut EntityGrid, &Teams), With<Node>>,
) -> OpImplResult {
    if !matches!(node_op, NodeOp::ReadyToGo) {
        Err(OpError::MismatchedOpSystem)?;
    }

    let mut metadata = Metadata::new();

    let (_, OnTeam(player_team), InNode(node_id), is_ready_to_go) =
        players.get(player).critical()?;
    if *is_ready_to_go {
        Err("You are already marked as ready".invalid())?;
    }
    let (access_point_loading_rule, mut grid, teams) = nodes.get_mut(*node_id).critical()?;
    let relevant_teams = match access_point_loading_rule {
        AccessPointLoadingRule::Staggered => vec![*player_team],
        AccessPointLoadingRule::Simultaneous => teams.0.clone(),
    };
    let has_loaded_access_point = access_points
        .iter()
        .any(|(id, OnTeam(team), access_point)| {
            grid.contains_key(id) && team == player_team && access_point.card.is_some()
        });
    if !has_loaded_access_point {
        Err("Must load an access point first".invalid())?;
    }

    let relevant_teams_are_ready =
        players
            .iter()
            .all(|(iter_player, OnTeam(team), _, ready_to_go)| {
                !relevant_teams.contains(team) || *ready_to_go || iter_player == player
            });
    metadata
        .put(key::ALL_TEAM_MEMBERS_READY, relevant_teams_are_ready)
        .critical()?;

    if relevant_teams_are_ready {
        let relevant_access_points: Vec<(Entity, Option<Entity>)> = access_points
            .iter()
            .filter(|(id, OnTeam(team), _)| player_team == team && grid.contains_key(*id))
            .map(|(id, _, access_point)| (id, access_point.card))
            .collect();
        for (player_id, OnTeam(team), _, _) in players.iter() {
            if relevant_teams.contains(team) {
                commands.entity(player_id).remove::<IsReadyToGo>();
            }
        }
        for (node_piece, card_id) in relevant_access_points.into_iter() {
            card_id
                .and_then(|card_id| {
                    let card_q = get_assert!(card_id, cards)?;
                    let mut ap_commands = commands.entity(node_piece);

                    ap_commands
                        .insert((
                            Curio::new_with_card(card_q.nickname_or_name(), card_id),
                            IsTapped::default(),
                            MovesTaken::default(),
                        ))
                        .remove::<AccessPoint>();

                    if !card_q.prevent_no_op() {
                        // Add No Op action
                        let mut new_actions = card_q.actions.clone();
                        new_actions.push(no_op_action.0.clone());

                        ap_commands.insert(Actions(new_actions));
                    }
                    Some(())
                })
                .unwrap_or_else(|| {
                    grid.remove_entity(node_piece);
                    // Leaving access points lying around seems bug prone, but so does despawning them?
                    // TODO Use play phase checks in ops, then remove the following line
                    commands.entity(node_piece).despawn()
                });
        }
        for team in relevant_teams {
            *team_phases
                .get_mut(team)
                .expect("Team should have team phase component") = TeamPhase::Play;
        }
    } else {
        *players.get_mut(player).unwrap().3 = true;
    };

    Ok(metadata)
}

fn opsys_node_end_turn(
    In((player, node_op)): In<(Entity, NodeOp)>,
    mut nodes: Query<
        (
            AsDerefMut<CurrentTurn>,
            AsDerefMut<ActiveCurio>,
            AsDeref<Teams>,
        ),
        With<Node>,
    >,
    players: Query<(AsDerefCopied<OnTeam>, AsDerefCopied<InNode>), With<Player>>,
    mut pieces: Query<
        (
            Entity,
            AsDerefCopied<OnTeam>,
            AsDerefMut<IsTapped>,
            AsDerefMut<MovesTaken>,
        ),
        With<NodePiece>,
    >,
) -> OpImplResult {
    if !matches!(node_op, NodeOp::EndTurn) {
        Err(OpError::MismatchedOpSystem)?;
    }

    let (player_team, node) = players.get(player).critical()?;
    let (mut current_turn, mut active_curio, teams) = nodes.get_mut(node).critical()?;

    if *current_turn.as_ref() != player_team {
        Err("Not this player's turn")?;
    }
    let mut metadata = Metadata::new();
    if let Some(id) = *active_curio {
        metadata.put(key::CURIO, id).critical()?;
    }
    active_curio.set_if_neq(None);
    let team_index = teams
        .iter()
        .position(|team_id| *team_id == player_team)
        .ok_or("Can't find this team".critical())?;
    *current_turn = teams[(team_index + 1) % teams.len()];

    // Gotta untap all player things
    let moved_pieces: HashMap<Entity, u32> = pieces
        .iter_mut()
        .filter_map(|(id, team, mut is_tapped, mut moves_taken)| {
            if team == player_team && (*is_tapped || *moves_taken > 0) {
                let old_moves_taken = *moves_taken;
                *moves_taken = 0;
                *is_tapped = false;
                Some((id, old_moves_taken))
            } else {
                None
            }
        })
        .collect();
    metadata.put(key::MOVED_PIECES, moved_pieces).critical()?;
    Ok(metadata)
}

fn opsys_telegraph_action(In((_, op)): In<(Entity, NodeOp)>) -> OpImplResult {
    if let NodeOp::TelegraphAction { .. } = op {
        let metadata = Metadata::new();
        Ok(metadata)
    } else {
        Err(OpError::MismatchedOpSystem)
    }
}

fn opsys_node_enter_battle(
    In((player_id, node_op)): In<(Entity, NodeOp)>,
    mut commands: Commands,
    res_asset_server: Res<AssetServer>,
    res_daddy_node: Res<Daddy<Node>>,
    res_reg_nodes: Res<Reg<NodeScene>>,
    q_nodes: Query<&Node>,
) -> OpImplResult {
    if let NodeOp::EnterNode(node_sid) = node_op {
        let node_already_open = q_nodes.iter().any(|Node(sid)| *sid == node_sid);
        // TODO check if player is already in a node. Cleanup?
        // Add ability to configure node behavior: Autoclose when last player leaves, or leave it open.
        // Also configurable: If a player leaves and node is not closed, should I restart the node?
        // Should I prompt player upon leaving a node if they would like to save their progress?
        if !node_already_open {
            if let Some(path) = res_reg_nodes.get(node_sid.to_string().as_str()) {
                let node_asset_handle: Handle<DynamicScene> = res_asset_server.load(path);
                commands
                    .spawn(node_asset_handle)
                    .set_parent(**res_daddy_node);
            } else {
                log::error!("Unable to find scene file for [{node_sid}] in the registry ")
            }
        }

        commands.entity(player_id).insert(EnteringNode(node_sid));
        Ok(default())
    } else {
        Err(OpError::MismatchedOpSystem)
    }
}

fn opsys_node_quit_battle(
    In((player_id, node_op)): In<(Entity, NodeOp)>,
    mut commands: Commands,
    q_node: Query<(AsDerefCopied<Parent>, &Node, &TeamStatus)>,
    mut q_player: Query<(&InNode, &OnTeam, &mut PlayedCards, &mut QuestStatus), With<Player>>,
    q_ncp_players: Query<(Entity, &InNode), (With<Player>, With<Ncp>)>,
    q_claimed_pickup: Query<(&Pickup, &Claimed)>,
) -> OpImplResult {
    if let NodeOp::QuitNode(node_sid) = node_op {
        // TODO When the player is able to join multiple games and leave midway through,
        // we'll need to find the node that was actually quit.
        let (&InNode(node_id), OnTeam(team_id), mut played_cards, mut quest_status) =
            q_player.get_mut(player_id).invalid()?;
        // ASSUMES THAT THE NODE HAS NO PARENTS OTHER THAN THE SCENE
        let (node_scene_id, node, team_status) = q_node.get(node_id).invalid()?;
        if node.0 != node_sid {
            Err("The player is not in that node".invalid())?;
        }
        let pickups: Vec<Pickup> = q_claimed_pickup
            .iter()
            .filter(|(_, claimed)| claimed.player == player_id && claimed.node_id == node_id)
            .map(|(pickup, _)| pickup.clone())
            .collect();
        let victory_status = team_status
            .get(team_id)
            .ok_or("Couldn't find team status".invalid())?;

        let mut metadata = Metadata::new();
        match victory_status {
            VictoryStatus::Victory | VictoryStatus::PerfectVictory => {
                let first_victory = !quest_status.is_node_done(&node_sid);
                metadata.put(key::FIRST_VICTORY, first_victory).invalid()?;
                if first_victory {
                    quest_status.record_node_done(&node_sid);
                }
            },
            _ => {},
        }

        let node_still_in_use = q_ncp_players
            .iter()
            .any(|(ncp_id, &InNode(ncp_node_id))| ncp_id != player_id && ncp_node_id == node_id);

        metadata.put(key::NODE_ID, node_id).invalid()?;
        metadata.put_nonempty(key::PICKUPS, &pickups).invalid()?;
        metadata
            .put(key::VICTORY_STATUS, victory_status)
            .invalid()?;
        metadata
            .put(key::CLOSING_NODE, !node_still_in_use)
            .invalid()?;
        metadata
            .put(key::RETURNED_CARDS, played_cards.clear_location(node_id))
            .critical()?;

        if !node_still_in_use {
            commands.entity(node_scene_id).despawn_recursive();
        }

        commands.entity(player_id).remove::<(InNode, OnTeam)>();
        Ok(metadata)

        // Check victory status, if victorious, update quest status
        // Hooks for quit? (On victory this will likely at least trigger dialog)
        // Check for claimed pickups, and handle these as determined by node config
        // Remove InNode if player is in this node.
        // If no non-computer players left in Node, despawn node.
    } else {
        Err(OpError::MismatchedOpSystem)
    }
}

fn opsys_node_undo(
    In((player_id, node_op)): In<(Entity, NodeOp)>,
    mut commands: Commands,
    q_player: Query<(&OnTeam, &InNode), With<Player>>,
    mut q_node: Query<(&mut EntityGrid, AsDerefMut<ActiveCurio>), With<Node>>,
    mut q_team: Query<AsDerefMut<NodeUndoStack>, With<Team>>,
    mut q_curio: Query<(AsDerefMut<MovesTaken>, AsDerefMut<IsTapped>), With<Curio>>,
    mut q_curio_effects: Query<(AsDerefMut<MaximumSize>, AsDerefMut<MovementSpeed>), With<Curio>>,
) -> OpImplResult {
    if !matches!(node_op, NodeOp::Undo) {
        return Err(OpError::MismatchedOpSystem);
    }
    let (&OnTeam(team_id), &InNode(node_id)) = q_player.get(player_id).invalid()?;
    let mut undo_queue = q_team.get_mut(team_id).invalid()?;
    if undo_queue.len() == 0 {
        Err("Not able to undo any more".invalid())?;
    }
    let (mut grid, mut active_curio) = q_node.get_mut(node_id).critical()?;
    let mut undo_metadata = Metadata::new();
    undo_metadata.put(key::NODE_ID, node_id).invalid()?;
    for op_to_undo in undo_queue.drain(..).rev() {
        if let Ok(metadata) = op_to_undo.result() {
            match op_to_undo.op() {
                NodeOp::ActivateCurio { .. } => {
                    if let Some(curio_id) = *active_curio {
                        undo_metadata.put(key::CURIO, curio_id).critical()?;
                        *active_curio = None;
                    }
                },
                NodeOp::MoveActiveCurio { .. } => {
                    let curio_id = metadata.get_required(key::CURIO).critical()?;
                    let (mut moves_taken, mut is_tapped) = q_curio.get_mut(curio_id).critical()?;
                    if let Some(dropped_square) =
                        metadata.get_optional(key::DROPPED_SQUARE).critical()?
                    {
                        grid.push_back(dropped_square, curio_id);
                    }
                    undo_metadata.put(key::CURIO, curio_id).critical()?;
                    grid.pop_front(curio_id);

                    if let Some(pickup_id) = metadata.get_optional(key::PICKUP_ID).critical()? {
                        // TODO Configurable return pick-up (cq should, nf should not)
                        let target_pt = metadata.get_required(key::TARGET_POINT).critical()?;
                        grid.put_item(target_pt, pickup_id);
                        commands.entity(pickup_id).remove::<Claimed>();
                    }
                    active_curio.set_if_neq(Some(curio_id)); // In case something goes wrong before full undo can occur
                    is_tapped.set_if_neq(false);
                    *moves_taken -= 1;
                },
                NodeOp::PerformCurioAction { .. } => {
                    let curio_id = metadata.get_required(key::CURIO).critical()?;
                    let (_, mut is_tapped) = q_curio.get_mut(curio_id).critical()?;
                    let skipped_activation =
                        metadata.get_required(key::SKIPPED_ACTIVATION).critical()?;
                    undo_metadata.put(key::CURIO, curio_id).critical()?;
                    if !skipped_activation {
                        active_curio.set_if_neq(Some(curio_id));
                    } else {
                        active_curio.set_if_neq(None);
                    }
                    is_tapped.set_if_neq(false);

                    if let Some(effects) = metadata.get_optional(key::EFFECTS).critical()? {
                        // More than invalid, but not critical
                        ActionEffect::revert_effects(
                            effects,
                            &mut grid,
                            q_curio_effects.as_query_lens(),
                        )
                        .critical()?;
                    }
                    if let Some(self_effects) =
                        metadata.get_optional(key::SELF_EFFECTS).invalid()?
                    {
                        ActionEffect::revert_effects(
                            self_effects,
                            &mut grid,
                            q_curio_effects.as_query_lens(),
                        )
                        .critical()?;
                    }
                },
                _ => {
                    log::error!("Invalid op in undo queue: {op_to_undo:?}");
                },
            }
        }
    }
    Ok(undo_metadata)
}
