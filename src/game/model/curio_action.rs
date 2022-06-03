use super::keys::curio_action_keys as keys;
use super::super::error::{ErrorMsg as _, Result};
use super::super::Metadata;
use super::SpritePoint;
use crate::{Node, Sprite, Point};
use getset::{CopyGetters, Getters};
use std::{num::NonZeroUsize, ops::RangeInclusive};
use typed_key::{typed_key, Key};

mod standard_curio_actions;

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
        sprite: Sprite,
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
            SACondition::Size(range) => range.contains(&node.sprite_len(curio_key)),
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
        let deleted_sprite = metadata.get(keys::DELETED_SPRITE)?;
        let damages = metadata.get_or_default(keys::DAMAGES)?;
        /*let deleted_sprites = metadata.get_or_default(keys::DELETED_SPRITES)?;
        for (key, Sprite) in deleted_sprite.into_iter() {

        }*/
        if let Some((key, spr)) = deleted_sprite {
            log::debug!("Tell me about things {}", key);
            let head = damages
                .iter()
                .find(|dropped_square| dropped_square.0 == key)
                .map(|dropped_square| Ok(dropped_square.1))
                .unwrap_or_else(|| {
                    "Unable to restore deleted sprite, no squares were dropped from it?"
                        .fail_critical()
                })?;
            unsafe {
                log::debug!("Tell me about things {}", key);
                // I am just returning an item that was removed
                node.return_sprite_with_key(key, head, spr);
            }
        }
        for dropped_square in damages.iter() {
            node.with_curio_mut(dropped_square.0, |mut curio| {
                curio.grow_back(dropped_square.1);
            })
            .ok_or_else(|| {
                format!(
                    "Could not find curio to undo dropped square {:?}",
                    dropped_square
                )
                .fail_critical_msg()
            })?;
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
                let mut metadata = Metadata::new();

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
                        metadata.put(keys::DAMAGES, &dropped_pts)?;
                        let deleted_sprite_opt = Some(key).zip(deleted_curio);
                        metadata.put_optional(keys::DELETED_SPRITE, &deleted_sprite_opt)?;
                        if let Some(deleted_sprite) = deleted_sprite_opt {
                            let mut deleted_sprites = metadata.get_or_default(keys::DELETED_SPRITES)?;
                            deleted_sprites.push(deleted_sprite);
                            metadata.put(keys::DELETED_SPRITES, &deleted_sprites)?;
                        }
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
                Ok(metadata)
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
