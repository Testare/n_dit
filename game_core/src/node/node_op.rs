mod node_opv2;
use std::borrow::Cow;

use bevy::ecs::query::WorldQuery;
use thiserror::Error;

use super::{
    IsTapped, MovesTaken, OnTeam,
};
use crate::card::{
    Actions, Card, Description, MaximumSize, MovementSpeed,
};
use crate::common::metadata::MetadataErr;
use crate::op::OpSubtype;
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
    #[error("Already ready")]
    AlreadyReady,
    #[error("Cannot be ready")]
    CannotBeReady,
}

impl OpSubtype for NodeOp {
    type Error = NodeOpError;
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

