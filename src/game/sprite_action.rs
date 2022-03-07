use crate::{Node, Piece, Point};
use getset::{CopyGetters, Getters};
use std::{num::NonZeroUsize, ops::RangeInclusive};

mod standard_sprite_actions;

pub use standard_sprite_actions::StandardSpriteAction;

// TODO look into making this a trait instead?
#[derive(Debug, CopyGetters, Getters)]
pub struct SpriteAction<'a> {
    #[get = "pub"]
    name: &'a str,
    #[get_copy = "pub"]
    genre: SpriteActionGenre,
    #[get_copy = "pub"]
    range: Option<NonZeroUsize>,
    effect: SAEffect,
    targets: Vec<Target>,
    conditions: Vec<SACondition>,
}

#[derive(Clone, Copy, Debug)]
pub enum SpriteActionGenre {
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
    fn met(&self, node: &Node, sprite_key: usize, target_pt: Point) -> bool {
        match self {
            SACondition::Size(range) => range.contains(&node.piece_len(sprite_key)),
            SACondition::TargetSize(range) => node
                .with_sprite_at(target_pt, |target| range.contains(&target.size()))
                .unwrap_or(false),
            SACondition::TargetMaxSize(range) => node
                .with_sprite_at(target_pt, |target| range.contains(&target.max_size()))
                .unwrap_or(false),
            _ => unimplemented!("TODO implement other conditions"),
        }
    }
}

// TODO enumset?
#[derive(Clone, Copy, Debug)]
pub enum Target {
    Ally = 0,
    // Area,
    _ClosedSquare,
    _EmptySquare,
    Enemy,
    _Itself,
}

impl SpriteAction<'_> {
    // Unsafe: Assumes NODE exists
    // In the future, require a GameState with a verified node?
    // Or perhaps just Node
    pub fn apply(
        &self,
        node: &mut Node,
        sprite_key: usize,
        target_pt: Point,
    ) -> Result<(), SpriteActionError> {
        if let Some(_target_type) = self
            .targets
            .iter()
            .find(|target| target.matches(node, sprite_key, target_pt))
        {
            if self
                .conditions
                .iter()
                .all(|condition| condition.met(node, sprite_key, target_pt))
            {
                match self.effect {
                    SAEffect::DealDamage(dmg) => {
                        let _: Option<Piece> = node
                            .with_sprite_at_mut(target_pt, |target| target.take_damage(dmg))
                            .ok_or(SpriteActionError::SpriteSpecificEffectOnNonSpriteTarget)?;
                    }
                    SAEffect::IncreaseMaxSize { amount, bound } => {
                        node.with_sprite_at_mut(target_pt, |mut target| {
                            target.increase_max_size(amount, bound)
                        })
                        .ok_or(SpriteActionError::SpriteSpecificEffectOnNonSpriteTarget)?;
                    }
                    _ => unimplemented!("Not implemented yet!"),
                }
                Ok(())
            } else {
                Err(SpriteActionError::ConditionNotMet)
            }
        } else {
            Err(SpriteActionError::InvalidTarget)
        }
    }
}

impl Target {
    fn matches(&self, node: &Node, sprite_key: usize, target_pt: Point) -> bool {
        match self {
            Self::Enemy => {
                let source_team = node.with_sprite(sprite_key, |sprite| sprite.team());
                let target_team = node.with_sprite_at(target_pt, |sprite| sprite.team());
                source_team.is_some() && target_team.is_some() && source_team != target_team
            }
            Self::Ally => {
                // TODO shouldn't be able to target self.
                let source_team = node.with_sprite(sprite_key, |sprite| sprite.team());
                let target_team = node.with_sprite_at(target_pt, |sprite| sprite.team());
                source_team.is_some() && target_team.is_some() && source_team == target_team
            }
            _ => unimplemented!("Target {:?} not implemented yet", self),
        }
    }
}

#[non_exhaustive]
pub enum SpriteActionError {
    InvalidTarget,
    ConditionNotMet,
    SpriteSpecificEffectOnNonSpriteTarget,
}
