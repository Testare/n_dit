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
pub enum SAEffect {
    DealDamage(usize),
    IncreaseMaxSize {
        amount: usize,
        bound: Option<NonZeroUsize>,
    },
    IncreaseMovementSpeed {
        amount: usize,
        bound: Option<NonZeroUsize>,
    },
    Recover {
        amount: usize,
        bound: Option<NonZeroUsize>,
    },
    Create {
        piece: Piece,
    },
    OpenSquare,
    CloseSquare,
}

#[derive(Debug)]
pub enum SACondition {
    Size(RangeInclusive<usize>),
    TargetSize(RangeInclusive<usize>),
    TargetMaxSize(RangeInclusive<usize>),
    Uses(usize),
    UsesPerTarget(usize),
}

impl SACondition {
    fn met(&self, node: &Node, sprite_key: usize, _target_pt: Point) -> bool {
        match self {
            SACondition::Size(range) => range.contains(&node.piece_len(sprite_key)),
            _ => unimplemented!("TODO implement other conditions"),
        }
    }
}

// TODO enumset?
#[derive(Clone, Copy, Debug)]
pub enum Target {
    Ally = 0,
    // Area,
    ClosedSquare,
    EmptySquare,
    Enemy,
    Itself,
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
                        let target_key = node
                            .piece_key_at(target_pt)
                            .ok_or(SpriteActionError::DealingDamageToNoPiece)?; // Assume it to be valid if target checks out
                        let mut grid = node.grid_mut();
                        grid.pop_back_n(target_key, dmg);
                        // Win condition check
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
    // UNSAFE assumes presence of node
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

pub enum SpriteActionError {
    InvalidTarget,
    ConditionNotMet,
    DealingDamageToNoPiece,
}
