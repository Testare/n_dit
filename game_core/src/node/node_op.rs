use bevy::ecs::query::WorldQuery;

use super::{NodePiece, PlayedCards};
use crate::card::{Actions, Card, Deck, Description, MaximumSize, MovementSpeed};
use crate::node::AccessPoint;
use crate::player::Player;
use crate::prelude::*;

const ACCESS_POINT_DISPLAY_ID: &'static str = "env:access_point";

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

pub fn access_point_actions(
    mut commands: Commands,
    mut ops: EventReader<Op<NodeOp>>,
    cards: Query<CardInfo>,
    mut access_points: Query<(&mut AccessPoint, &mut NodePiece)>,
    mut players: Query<(&mut PlayedCards, &Deck), With<Player>>,
) {
    for command in ops.iter() {
        if let Ok((mut played_cards, deck)) = players.get_mut(command.player()) {
            match command.op() {
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
