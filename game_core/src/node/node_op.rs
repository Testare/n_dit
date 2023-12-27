use std::borrow::Cow;

use bevy::ecs::query::WorldQuery;
use bevy::reflect::TypePath;

use crate::card::{
    Action, Actions, Card, CardDefinition, Deck, Description, MaximumSize, MovementSpeed,
};
use crate::configuration::PlayerConfiguration;
use crate::node::{
    key, AccessPoint, AccessPointLoadingRule, ActiveCurio, Curio, CurrentTurn, InNode, IsReadyToGo,
    IsTapped, MovesTaken, NoOpAction, Node, NodePiece, OnTeam, Pickup, PlayedCards, Team,
    TeamPhase, TeamStatus, Teams, VictoryStatus,
};
use crate::op::{CoreOps, Op, OpError, OpErrorUtils, OpImplResult, OpRegistrar};
use crate::player::Player;
use crate::prelude::*;

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
}

#[derive(Debug, WorldQuery)]
pub struct CardInfo {
    card: &'static Card,
    description: Option<&'static Description>,
    speed: Option<&'static MovementSpeed>,
    size: Option<&'static MaximumSize>,
    actions: Option<&'static Actions>,
}

#[derive(Debug, WorldQuery)]
#[world_query(mutable)]
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
            .register_op(opsys_telegraph_action);
    }

    fn system_index(&self) -> usize {
        log::debug!("NodeOp: {self:?}");
        match self {
            Self::MoveActiveCurio { .. } => 0,
            Self::PerformCurioAction { .. } => 1,
            Self::ActivateCurio { .. } => 2,
            Self::LoadAccessPoint { .. } => 3,
            Self::UnloadAccessPoint { .. } => 3,
            Self::ReadyToGo { .. } => 4,
            Self::EndTurn => 5,
            Self::TelegraphAction { .. } => 6,
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
    ast_cards: Res<Assets<CardDefinition>>,
    mut commands: Commands,
    cards: Query<CardInfo>,
    mut access_points: Query<(&mut AccessPoint, &mut NodePiece)>,
    mut players: Query<(&mut PlayedCards, &Deck), With<Player>>,
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
    let (mut played_cards, deck) = players.get_mut(player).critical()?;
    if access_point.card.is_none() && next_card_id.is_none() {
        Err("Access point is already unloaded".invalid())?;
    }
    if access_point.card == next_card_id {
        Err("That is already loaded".invalid())?;
    }
    if let Some(card) = access_point.card {
        metadata.put(key::CARD, card).critical()?;
        let card_count = played_cards
            .get_mut(&card)
            .ok_or("Unloading card that wasn't played".critical())?;
        *card_count -= 1;
    }

    let mut access_point_commands = commands.entity(access_point_id);
    if let Some(next_card_id) = next_card_id {
        if !played_cards.can_be_played(deck, next_card_id) {
            Err("Already played all of those".invalid())?;
        }
        let card_info = cards
            .get(next_card_id)
            .map_err(|_| "Cannot find that card".invalid())?;
        let card_def = ast_cards
            .get(card_info.card.definition())
            .ok_or("Card definition is not loaded".invalid())?; // Not really invalid?

        *played_cards.entry(next_card_id).or_default() += 1;
        node_piece.set_display_id(card_info.card.card_name().to_owned());

        access_point_commands.insert((
            Description::new(card_def.description().to_owned()),
            MovementSpeed(card_def.movement_speed()),
            MaximumSize(card_def.max_size()),
            Actions(card_def.actions().clone()),
        ));
    } else {
        node_piece.set_display_id(ACCESS_POINT_DISPLAY_ID.to_owned());
        access_point_commands.remove::<(Description, MovementSpeed, MaximumSize, Actions)>();
    };
    access_point.card = next_card_id;

    Ok(metadata)
}

fn opsys_node_ready(
    In((player, node_op)): In<(Entity, NodeOp)>,
    no_op_action: Res<NoOpAction>,
    ast_cards: Res<Assets<CardDefinition>>,
    mut commands: Commands,
    cards: Query<&Card>,
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
                    let card = get_assert!(card_id, cards)?;
                    let card_base = ast_cards.get(card.definition())?;
                    let mut ap_commands = commands.entity(node_piece);

                    ap_commands
                        .insert((
                            Curio::new_with_card(card.card_name(), card_id),
                            IsTapped::default(),
                            MovesTaken::default(),
                        ))
                        .remove::<AccessPoint>();

                    if !card_base.prevent_no_op() {
                        // Add No Op action
                        let mut new_actions = card_base.actions().clone();
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
