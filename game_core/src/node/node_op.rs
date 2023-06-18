use bevy::ecs::query::WorldQuery;

use super::{
    AccessPointLoadingRule, InNode, Node, NodePiece, Team, OnTeam, PlayedCards, IsReadyToGo,
    TeamPhase, Teams,
};
use crate::card::{Actions, Card, Deck, Description, MaximumSize, MovementSpeed};
use crate::node::{AccessPoint, Curio};
use crate::player::Player;
use crate::prelude::*;

const ACCESS_POINT_DISPLAY_ID: &'static str = "env:access_point";

#[derive(Debug)]
pub enum NodeOp {
    LoadAccessPoint {
        access_point_id: Entity,
        card_id: Entity,
    },
    UnloadAccessPoint {
        access_point_id: Entity,
    },
    ReadyToGo,
}

#[derive(WorldQuery)]
pub struct CardInfo {
    card: &'static Card,
    description: Option<&'static Description>,
    speed: Option<&'static MovementSpeed>,
    size: Option<&'static MaximumSize>,
    actions: Option<&'static Actions>,
}

// TODO Ready to go when it isn't your turn
pub fn ready_to_go_ops(
    mut commands: Commands,
    mut ops: EventReader<Op<NodeOp>>,
    cards: Query<&Card>,
    mut players: Query<(Entity, &OnTeam, &InNode, &mut IsReadyToGo), With<Player>>,
    mut team_phases: Query<&mut TeamPhase, With<Team>>,
    access_points: Query<(Entity, &OnTeam, &AccessPoint), With<NodePiece>>,
    mut nodes: Query<(&AccessPointLoadingRule, &mut EntityGrid, &Teams), With<Node>>,
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
                    if valid_op {
                        let relevant_teams_are_ready = players.iter().all(
                            |(iter_player, OnTeam(team), _, IsReadyToGo(ready_to_go))| {
                                !relevant_teams.contains(team)
                                    || *ready_to_go
                                    || iter_player == *player
                            },
                        );
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
                            for (node_piece, card) in relevant_access_points.into_iter() {
                                if let Some(card_id) = card {
                                    let card_name =
                                        cards.get(card_id).expect("card should exist").card_name();

                                    commands
                                        .entity(node_piece)
                                        .insert(Curio::new_with_card(card_name, card_id))
                                        .remove::<AccessPoint>();
                                } else {
                                    let piece_len = grid.len_of(node_piece);
                                    grid.pop_back_n(node_piece, piece_len);
                                    commands.entity(node_piece).despawn()
                                }
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
                    }
                }
            }
        }
    }
}

pub fn access_point_ops(
    mut commands: Commands,
    mut ops: EventReader<Op<NodeOp>>,
    cards: Query<CardInfo>,
    mut access_points: Query<(&mut AccessPoint, &mut NodePiece)>,
    mut players: Query<(&mut PlayedCards, &Deck), With<Player>>,
) {
    for node_op in ops.iter() {
        if let Ok((mut played_cards, deck)) = players.get_mut(node_op.player()) {
            match node_op.op() {
                NodeOp::LoadAccessPoint {
                    access_point_id,
                    card_id,
                } => {
                    if let Ok((mut access_point, mut node_piece)) =
                        access_points.get_mut(*access_point_id)
                    {
                        if !played_cards.can_be_played(deck, *card_id) {
                            continue;
                        }
                        *played_cards.entry(*card_id).or_default() += 1;
                        let mut access_point_commands = commands.entity(*access_point_id);

                        if let Some(card_id) = access_point.card {
                            let played_count = played_cards.entry(card_id).or_default();
                            *played_count = played_count.saturating_sub(1);

                            access_point_commands
                                .remove::<(Description, MovementSpeed, MaximumSize, Actions)>();
                        }
                        access_point.card = Some(*card_id);
                        if let Ok(card_info) = cards.get(*card_id) {
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
                        }
                    }
                },
                NodeOp::UnloadAccessPoint { access_point_id } => {
                    if let Ok((mut access_point, mut node_piece)) =
                        access_points.get_mut(*access_point_id)
                    {
                        if let Some(card_count) = access_point
                            .card
                            .and_then(|card_id| played_cards.get_mut(&card_id))
                        {
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
                        }
                    }
                },
                _ => {},
            }
        }
    }
}
