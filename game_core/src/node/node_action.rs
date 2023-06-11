use bevy::ecs::query::WorldQuery;

use crate::card::Card;
use crate::prelude::*;
use crate::{AccessPoint, Actions, Description, MaximumSize, MovementSpeed};

pub enum NodeAction {
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
    mut actions: EventReader<Act<NodeAction>>,
    cards: Query<CardInfo>,
    mut access_points: Query<&mut AccessPoint>,
) {
    for command in actions.iter() {
        match command.action() {
            NodeAction::LoadAccessPoint {
                access_point_id,
                card_id,
            } => {
                if let Ok(mut access_point) = access_points.get_mut(*access_point_id) {
                    let mut access_point_commands = commands.entity(*access_point_id);

                    if access_point.card.is_some() {
                        access_point_commands
                            .remove::<(Description, MovementSpeed, MaximumSize, Actions)>();
                    }
                    access_point.card = Some(*card_id);
                    if let Ok(card_info) = cards.get(*card_id) {
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
            NodeAction::UnloadAccessPoint { access_point_id } => {
                if let Ok(mut access_point) = access_points.get_mut(*access_point_id) {
                    let mut access_point_commands = commands.entity(*access_point_id);

                    if access_point.card.is_some() {
                        access_point_commands
                            .remove::<(Description, MovementSpeed, MaximumSize, Actions)>();
                    }
                    access_point.card = None;
                }
            },
            _ => {},
        }
    }
}
