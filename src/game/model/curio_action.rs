use super::super::error::{ErrorMsg as _, Result};
use super::super::{DroppedSquare, Metadata, NodeChangeMetadata};
use super::node::node_change_keys;
use crate::{Node, Piece, Point};
use getset::{CopyGetters, Getters};
use std::{num::NonZeroUsize, ops::RangeInclusive};
use typed_key::{typed_key, Key};

mod standard_curio_actions;

const DROPPED_SQUARES: Key<Vec<DroppedSquare>> = typed_key!("droppedSquares");

pub use standard_curio_actions::StandardCurioAction;

// TODO look into making this a trait instead?
#[derive(Debug, CopyGetters, Getters)]
pub struct CurioAction<'a> {
    #[get = "pub"]
    name: &'a str,
    #[get_copy = "pub"]
    genre: CurioActionGenre,
    #[get_copy = "pub"]
    range: Option<NonZeroUsize>,
    effect: SAEffect,
    targets: Vec<Target>,
    conditions: Vec<SACondition>,
}

#[derive(Clone, Copy, Debug)]
pub enum CurioActionGenre {
    Attack = 0,
    Support = 1,
}

#[derive(Debug)]
#[non_exhaustive]
pub enum SAEffect {
    DealDamage(usize),
    IncreaseMaxSize {
        amount: usize,
        bound: Option<NonZeroUsize>,
    },
    _IncreaseMovementSpeed {
        amount: usize,
        bound: Option<NonZeroUsize>,
    },
    _Recover {
        amount: usize,
        bound: Option<NonZeroUsize>,
    },
    _Create {
        piece: Piece,
    },
    _OpenSquare,
    _CloseSquare,
}

#[derive(Debug)]
#[non_exhaustive]
pub enum SACondition {
    Size(RangeInclusive<usize>),
    TargetSize(RangeInclusive<usize>),
    TargetMaxSize(RangeInclusive<usize>),
    _Uses(usize),
    _UsesPerTarget(usize),
}

impl SACondition {
    fn met(&self, node: &Node, curio_key: usize, target_pt: Point) -> bool {
        match self {
            SACondition::Size(range) => range.contains(&node.piece_len(curio_key)),
            SACondition::TargetSize(range) => node
                .with_curio_at(target_pt, |target| range.contains(&target.size()))
                .unwrap_or(false),
            SACondition::TargetMaxSize(range) => node
                .with_curio_at(target_pt, |target| range.contains(&target.max_size()))
                .unwrap_or(false),
            _ => unimplemented!("TODO implement other conditions"),
        }
    }
}

// TODO enumset?
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Target {
    Ally = 0,
    // Area,
    _ClosedSquare,
    _EmptySquare,
    Enemy,
    _Itself,
}

impl CurioAction<'_> {
    pub fn unapply(
        &self,
        node: &mut Node,
        curio_key: usize,
        target_pt: Point,
        metadata: &Metadata,
    ) -> Result<()> {
        let metadata = NodeChangeMetadata::from(metadata)?;
        if let Some((key, Piece::Program(spr))) = metadata.deleted_piece() {
            let head = metadata
                .dropped_squares()
                .iter()
                .find(|dropped_square| dropped_square.0 == *key)
                .map(|dropped_square| Ok(dropped_square.1))
                .unwrap_or_else(|| {
                    "Unable to restore deleted piece, no squares were dropped from it?"
                        .fail_critical()
                })?;
            unsafe {
                // I am just returning an item that was removed
                node.return_piece_with_key(*key, head, Piece::Program(spr.clone()));
            }
        }
        Ok(())
    }
    pub fn apply(&self, node: &mut Node, curio_key: usize, target_pt: Point) -> Result<Metadata> {
        if let Some(_target_type) = self
            .targets
            .iter()
            .find(|target| target.matches(node, curio_key, target_pt))
        {
            if self
                .conditions
                .iter()
                .all(|condition| condition.met(node, curio_key, target_pt))
            {
                let mut metadata = NodeChangeMetadata::for_team(node.active_team());
                let mut metadata2 = Metadata::new();

                match self.effect {
                    SAEffect::DealDamage(dmg) => {
                        let (key, (dropped_pts, deleted_curio)) = node
                            .with_curio_at_mut(target_pt, |target| {
                                (target.key(), target.take_damage(dmg))
                            })
                            .ok_or_else(|| {
                                "Invalid target for damage: Target must be a curio"
                                    .fail_reversible_msg()
                            })?;
                        metadata2.put(DROPPED_SQUARES, &dropped_pts)?;
                        metadata = metadata
                            .with_dropped_squares(dropped_pts)
                            .with_deleted_piece(Some(key).zip(deleted_curio));
                    }
                    SAEffect::IncreaseMaxSize { amount, bound } => {
                        node.with_curio_at_mut(target_pt, |mut target| {
                            target.increase_max_size(amount, bound)
                        })
                        .ok_or_else(|| {
                            "Invalid target for increase size: Target must be a curio"
                                .fail_reversible_msg()
                        })?;
                    }
                    _ => unimplemented!("Not implemented yet!"),
                }
                metadata.to_metadata()
            } else {
                "Conditions not met".invalid()
            }
        } else {
            "Invalid target for action".invalid()
        }
    }

    pub fn can_target_enemy(&self) -> bool {
        self.targets.contains(&Target::Enemy)
    }
}

impl Target {
    fn matches(&self, node: &Node, curio_key: usize, target_pt: Point) -> bool {
        match self {
            Self::Enemy => {
                let source_team = node.with_curio(curio_key, |curio| curio.team());
                let target_team = node.with_curio_at(target_pt, |curio| curio.team());
                source_team.is_some() && target_team.is_some() && source_team != target_team
            }
            Self::Ally => {
                // TODO shouldn't be able to target self.
                let source_team = node.with_curio(curio_key, |curio| curio.team());
                let target_team = node.with_curio_at(target_pt, |curio| curio.team());
                source_team.is_some() && target_team.is_some() && source_team == target_team
            }
            _ => unimplemented!("Target {:?} not implemented yet", self),
        }
    }
}

#[non_exhaustive]
pub enum CurioActionError {
    InvalidTarget,
    ConditionNotMet,
    CurioSpecificEffectOnNonCurioTarget,
}
