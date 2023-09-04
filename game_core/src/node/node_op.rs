use bevy::ecs::query::{Has, WorldQuery};
use thiserror::Error;

use super::{
    key, AccessPointLoadingRule, ActiveCurio, CurrentTurn, InNode, IsReadyToGo, IsTapped,
    MovesTaken, NoOpAction, Node, NodePiece, OnTeam, Pickup, PlayedCards, PreventNoOp, Team,
    TeamPhase, Teams,
};
use crate::card::{
    Action, ActionEffect, ActionRange, Actions, Card, Deck, Description, MaximumSize,
    MovementSpeed, Prereqs,
};
use crate::common::metadata::MetadataErr;
use crate::node::{AccessPoint, Curio};
use crate::op::{OpResult, OpSubtype};
use crate::player::Player;
use crate::prelude::*;

const ACCESS_POINT_DISPLAY_ID: &'static str = "env:access_point";

#[derive(Clone, Copy, Debug)]
pub enum NodeOp {
    PerformCurioAction {
        action: Entity,
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
}

#[derive(Clone, Debug, Error)]
#[non_exhaustive]
pub enum NodeOpError {
    #[error("No curio is currently active")]
    NoActiveCurio,
    #[error("No access point")]
    NoSuchAction,
    #[error("No such card")]
    NoSuchCard,
    #[error("This piece doesn't have a movement speed")]
    NoMovementSpeed,
    #[error("No movement remains")]
    NoMovementRemains,
    #[error("This is not a valid target for this action")]
    InvalidTarget, // TODO include target type
    #[error("This action's requirements are not satisfied")]
    PrereqsNotSatisfied, // TODO include failed prereq
    #[error("Out of range")]
    OutOfRange,
    #[error("A glitch has occurred")]
    InternalError,
    #[error("Glitch occurred with metadata while performing op: {0}")]
    MetadataSerializationError(#[from] MetadataErr),
    #[error("Could not find access point")]
    NoAccessPoint,
    #[error("You can't play that card")]
    UnplayableCard,
    #[error("Nothing was accomplished")]
    NothingToDo,
    #[error("Not your turn")]
    NotYourTurn,
}

impl OpSubtype for NodeOp {
    type Error = NodeOpError;
}

#[derive(WorldQuery)]
pub struct CardInfo {
    card: &'static Card,
    description: Option<&'static Description>,
    speed: Option<&'static MovementSpeed>,
    size: Option<&'static MaximumSize>,
    actions: Option<&'static Actions>,
}

#[derive(WorldQuery)]
#[world_query(mutable)]
pub struct CurioQ {
    team: &'static OnTeam,
    tapped: &'static mut IsTapped,
    moves_taken: &'static mut MovesTaken,
    movement_speed: Option<&'static mut MovementSpeed>,
    max_size: Option<&'static mut MaximumSize>,
    actions: Option<&'static Actions>,
}

pub fn curio_ops(
    no_op_action: Res<NoOpAction>,
    mut ops: EventReader<Op<NodeOp>>,
    mut nodes: Query<(&mut EntityGrid, &CurrentTurn, &mut ActiveCurio), With<Node>>,
    players: Query<(&OnTeam, &InNode), With<Player>>,
    team_phases: Query<&TeamPhase, With<Team>>,
    mut curios: Query<CurioQ, With<Curio>>,
    pickups: Query<&Pickup>,
    actions: Query<
        (
            Option<&ActionEffect>,
            Option<&ActionRange>,
            Option<&Prereqs>,
        ),
        With<Action>,
    >,
    mut ev_results: EventWriter<OpResult<NodeOp>>,
) {
    for fullop @ Op { op, player } in ops.into_iter() {
        players.get(*player).ok().and_then(|(player_team, node)| {
            let (mut grid, current_turn, mut active_curio) = nodes.get_mut(**node).ok()?;
            if **player_team != **current_turn {
                return None;
            }
            if *team_phases.get(**player_team).ok()? != TeamPhase::Play {
                return None;
            }
            match op {
                NodeOp::ActivateCurio { curio_id } => {
                    let target_curio = curios.get(*curio_id).ok()?;
                    if **target_curio.team != **player_team || **target_curio.tapped {
                        return None;
                    }
                    if let Some(last_active) = **active_curio {
                        if last_active != *curio_id {
                            **curios.get_mut(last_active).ok()?.tapped = true;
                            **active_curio = Some(*curio_id);
                        } else {
                            return None;
                        }
                    } else {
                        **active_curio = Some(*curio_id);
                    }
                },
                NodeOp::MoveActiveCurio { dir } => {
                    let result = active_curio.ok_or(NodeOpError::NoActiveCurio).and_then(
                        |active_curio_id| {
                            let mut metadata = Metadata::default();
                            metadata.put(key::NODE_ID, **node)?;
                            metadata.put(key::CURIO, active_curio_id)?;
                            let mut curio_q = curios
                                .get_mut(active_curio_id)
                                .map_err(|_| NodeOpError::InternalError)?;
                            debug_assert!(!**curio_q.tapped, "a tapped curio was active");
                            let movement_speed =
                                **curio_q.movement_speed.ok_or(NodeOpError::NoMovementSpeed)?;
                            if movement_speed == **curio_q.moves_taken {
                                return Err(NodeOpError::NoMovementRemains);
                            }
                            let head = grid
                                .head(active_curio_id)
                                .ok_or(NodeOpError::InternalError)?;
                            let next_pt = head + *dir;
                            metadata.put(key::TARGET_POINT, next_pt)?;
                            if grid.square_is_closed(next_pt) {
                                return Err(NodeOpError::InvalidTarget);
                            }
                            if let Some(entity_at_pt) = grid.item_at(next_pt) {
                                if entity_at_pt == active_curio_id {
                                    // Curios can move onto their own squares
                                } else if let Ok(pickup) = pickups.get(entity_at_pt) {
                                    grid.remove_entity(entity_at_pt);
                                    metadata.put(key::PICKUP, pickup)?;
                                    log::debug!("Picked up: {:?}", pickup);
                                } else {
                                    return Err(NodeOpError::InvalidTarget);
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
                                )?;
                                grid.pop_back(active_curio_id);
                            }
                            let remaining_moves = movement_speed - **curio_q.moves_taken;

                            metadata.put(key::REMAINING_MOVES, &remaining_moves)?;
                            if movement_speed == **curio_q.moves_taken {
                                if curio_q
                                    .actions
                                    .as_ref()
                                    .map(|curio_actions| {
                                        curio_actions
                                            .iter()
                                            .find(|action| **action != **no_op_action)
                                            .is_none()
                                    })
                                    .unwrap_or(true)
                                {
                                    metadata.put(key::TAPPED, true)?;
                                    **curio_q.tapped = true;
                                    **active_curio = None;
                                }
                            }
                            Ok(metadata)
                        },
                    );
                    // TODO use actual results
                    ev_results.send(OpResult::new(fullop, result));
                },
                NodeOp::PerformCurioAction {
                    action: action_id,
                    curio,
                    target,
                } => {
                    if active_curio.is_some() && curio.is_some() && **active_curio != *curio {
                        return None; // NodeOpError::CurioMismatch
                    }
                    let node_op_result = active_curio
                        .or(*curio)
                        .ok_or(NodeOpError::NoActiveCurio)
                        .and_then(|curio_id| {
                            let mut curio_q = get_assert_mut!(curio_id, curios)
                                .ok_or(NodeOpError::InternalError)?;
                            debug_assert!(!**curio_q.tapped, "Active curio should not be tapped");
                            if !curio_q
                                .actions
                                .ok_or(NodeOpError::NoSuchAction)?
                                .contains(action_id)
                            {
                                return Err(NodeOpError::NoSuchAction);
                            }

                            let (effect, range, prereqs) = get_assert!(*action_id, actions)
                                .ok_or(NodeOpError::InternalError)?;
                            if let Some(range) = range {
                                if !range.in_range_of(grid.as_ref(), curio_id, *target) {
                                    return Err(NodeOpError::OutOfRange);
                                }
                            }
                            if let Some(Prereqs(prereqs)) = prereqs {
                                for prereq in prereqs {
                                    if !prereq.satisfied(&grid, curio_id, *target) {
                                        return Err(NodeOpError::PrereqsNotSatisfied);
                                    }
                                }
                            }
                            // TODO action metadata should be lower
                            let mut action_metadata = if let Some(effect) = effect {
                                effect.apply_effect(&mut grid, curio_id, *target)?
                            } else {
                                Default::default()
                            };
                            action_metadata.put(key::NODE_ID, **node)?;
                            **curio_q.tapped = true;
                            **active_curio = None;
                            Ok(action_metadata)
                        });
                    ev_results.send(OpResult::new(fullop, node_op_result));
                },
                _ => {},
            }

            Some(())
        });
    }
}

// TODO Ready to go when it isn't your turn
pub fn ready_to_go_ops(
    no_op_action: Res<NoOpAction>,
    mut commands: Commands,
    mut ops: EventReader<Op<NodeOp>>,
    cards: Query<(&Card, Option<&Actions>, Has<PreventNoOp>)>,
    mut players: Query<(Entity, &OnTeam, &InNode, &mut IsReadyToGo), With<Player>>,
    mut team_phases: Query<&mut TeamPhase, With<Team>>,
    access_points: Query<(Entity, &OnTeam, &AccessPoint), With<NodePiece>>,
    mut nodes: Query<(&AccessPointLoadingRule, &mut EntityGrid, &Teams), With<Node>>,
    mut ev_op_result: EventWriter<OpResult<NodeOp>>,
) {
    for op in ops.iter() {
        if let Op {
            player,
            op: NodeOp::ReadyToGo,
        } = op
        {
            if let Ok((_, OnTeam(player_team), InNode(node_id), IsReadyToGo(false))) =
                players.get(*player)
            {
                if let Ok((access_point_loading_rule, mut grid, teams)) = nodes.get_mut(*node_id) {
                    let relevant_teams = match access_point_loading_rule {
                        AccessPointLoadingRule::Staggered => vec![*player_team],
                        AccessPointLoadingRule::Simultaneous => teams.0.clone(),
                    };

                    let valid_op = access_points
                        .iter()
                        .any(|(id, OnTeam(team), access_point)| {
                            grid.contains_key(id)
                                && team == player_team
                                && access_point.card.is_some()
                        });
                    let result = if valid_op {
                        let mut metadata: Metadata = default();
                        let relevant_teams_are_ready = players.iter().all(
                            |(iter_player, OnTeam(team), _, IsReadyToGo(ready_to_go))| {
                                !relevant_teams.contains(team)
                                    || *ready_to_go
                                    || iter_player == *player
                            },
                        );
                        metadata.put(key::ALL_TEAM_MEMBERS_READY, relevant_teams_are_ready);
                        if relevant_teams_are_ready {
                            let relevant_access_points: Vec<(Entity, Option<Entity>)> =
                                access_points
                                    .iter()
                                    .filter_map(|(id, OnTeam(team), access_point)| {
                                        (player_team == team && grid.contains_key(id))
                                            .then(|| (id, access_point.card))
                                    })
                                    .collect();
                            for (player_id, OnTeam(team), _, _) in players.iter() {
                                if relevant_teams.contains(team) {
                                    commands.entity(player_id).remove::<IsReadyToGo>();
                                }
                            }
                            for (node_piece, card_id) in relevant_access_points.into_iter() {
                                card_id
                                    .and_then(|card_id| {
                                        let (card, card_actions, prevent_no_op) =
                                            get_assert!(card_id, cards)?;
                                        // Can be tapped

                                        let mut ap_commands = commands.entity(node_piece);

                                        ap_commands
                                            .insert((
                                                Curio::new_with_card(card.card_name(), card_id),
                                                IsTapped::default(),
                                                MovesTaken::default(),
                                            ))
                                            .remove::<AccessPoint>();

                                        if !prevent_no_op {
                                            // Add No Op action
                                            let mut new_actions = card_actions
                                                .cloned()
                                                .unwrap_or_else(|| Actions(vec![]));
                                            new_actions.0.push(**no_op_action);

                                            ap_commands.insert(new_actions);
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
                                    .expect("Team should have team phase component") =
                                    TeamPhase::Play;
                            }
                        } else {
                            players.get_mut(*player).unwrap().3 .0 = true;
                        };
                        Ok(metadata)
                    } else {
                        Err(NodeOpError::InternalError)
                    };
                    ev_op_result.send(OpResult::new(op, result));
                }
            }
        }
    }
}

pub fn end_turn_op(
    mut evr_node_ops: EventReader<Op<NodeOp>>,
    mut nodes: Query<
        (
            AsDerefMut<CurrentTurn>,
            AsDerefMut<ActiveCurio>,
            AsDeref<Teams>,
        ),
        With<Node>,
    >,
    mut players: Query<(AsDerefCopied<OnTeam>, AsDerefCopied<InNode>), With<Player>>,
    mut pieces: Query<
        (
            Entity,
            AsDerefCopied<OnTeam>,
            AsDerefMut<IsTapped>,
            AsDerefMut<MovesTaken>,
        ),
        With<NodePiece>,
    >,
    mut evw_op_results: EventWriter<OpResult<NodeOp>>,
) {
    for ev in evr_node_ops.iter() {
        if matches!(ev.op(), NodeOp::EndTurn) {
            // Potential future improvement: Check if there is an active curio that does not have the no_op action and prevent end_turn.
            let result = get_assert!(ev.player(), players)
                .ok_or(NodeOpError::InternalError)
                .and_then(|(player_team, node)| {
                    let (mut current_turn, mut active_curio, teams) =
                        get_assert_mut!(node, nodes).ok_or(NodeOpError::InternalError)?;
                    if *current_turn.as_ref() != player_team {
                        return Err(NodeOpError::NotYourTurn);
                    }
                    let mut metadata = Metadata::new();
                    if let Some(id) = *active_curio {
                        metadata.put(key::CURIO, id);
                    }
                    active_curio.set_if_neq(None);
                    let current_pos = teams
                        .iter()
                        .position(|team_id| *team_id == player_team)
                        .ok_or(NodeOpError::InternalError)?;
                    *current_turn = teams[(current_pos + 1) % teams.len()];
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
                    metadata.put(key::MOVED_PIECES, moved_pieces);
                    Ok(metadata)
                });
            evw_op_results.send(OpResult::new(ev, result));
        }
    }
}

pub fn access_point_ops(
    mut commands: Commands,
    mut ops: EventReader<Op<NodeOp>>,
    cards: Query<CardInfo>,
    mut access_points: Query<(&mut AccessPoint, &mut NodePiece)>,
    mut players: Query<(&mut PlayedCards, &Deck), With<Player>>,
    mut ev_op_result: EventWriter<OpResult<NodeOp>>,
) {
    for node_op in ops.iter() {
        if let Ok((mut played_cards, deck)) = players.get_mut(node_op.player()) {
            match node_op.op() {
                NodeOp::LoadAccessPoint {
                    access_point_id,
                    card_id,
                } => {
                    let op_result = access_points
                        .get_mut(*access_point_id)
                        .map_err(|_| NodeOpError::NoAccessPoint)
                        .and_then(|(mut access_point, mut node_piece)| {
                            if !played_cards.can_be_played(deck, *card_id) {
                                return Err(NodeOpError::UnplayableCard);
                            }
                            let card_info =
                                cards.get(*card_id).map_err(|_| NodeOpError::NoSuchCard)?;
                            *played_cards.entry(*card_id).or_default() += 1;
                            let mut access_point_commands = commands.entity(*access_point_id);

                            if let Some(card_id) = access_point.card {
                                let played_count = played_cards.entry(card_id).or_default();
                                *played_count = played_count.saturating_sub(1);

                                access_point_commands.remove::<(
                                    Description,
                                    MovementSpeed,
                                    MaximumSize,
                                    Actions,
                                )>();
                            }
                            access_point.card = Some(*card_id);
                            node_piece.set_display_id(card_info.card.display_id().clone());
                            if let Some(description) = card_info.description {
                                access_point_commands.insert(description.clone());
                            }
                            if let Some(speed) = card_info.speed {
                                access_point_commands.insert(speed.clone());
                            }
                            if let Some(size) = card_info.size {
                                access_point_commands.insert(size.clone());
                            }
                            if let Some(actions) = card_info.actions {
                                access_point_commands.insert(actions.clone());
                            }
                            Ok(Default::default())
                        });
                    ev_op_result.send(OpResult::new(node_op, op_result));
                },
                NodeOp::UnloadAccessPoint { access_point_id } => {
                    let op_result = access_points
                        .get_mut(*access_point_id)
                        .map_err(|_| NodeOpError::NoAccessPoint)
                        .and_then(|(mut access_point, mut node_piece)| {
                            if let Some((card_count, card_id)) =
                                access_point.card.and_then(|card_id| {
                                    played_cards.get_mut(&card_id).zip(Some(card_id))
                                })
                            {
                                let mut metadata = Metadata::new();
                                metadata.put(key::CARD, card_id);
                                *card_count = card_count.saturating_sub(1);
                                let mut access_point_commands = commands.entity(*access_point_id);
                                node_piece.set_display_id(ACCESS_POINT_DISPLAY_ID.to_owned());

                                if access_point.card.is_some() {
                                    access_point_commands.remove::<(
                                        Description,
                                        MovementSpeed,
                                        MaximumSize,
                                        Actions,
                                    )>();
                                }
                                access_point.card = None;
                                Ok(metadata)
                            } else {
                                Err(NodeOpError::NothingToDo)
                            }
                        });
                    ev_op_result.send(OpResult::new(node_op, op_result));
                },
                _ => {},
            }
        }
    }
}
