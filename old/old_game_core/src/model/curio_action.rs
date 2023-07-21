use super::super::error::{ErrorMsg as _, Result};
use super::super::Metadata;
use super::keys::curio_action_keys as keys;
use crate::assets::{ActionCondition, ActionDef, ActionEffect, ActionTarget};
use crate::{Node, Point};

type CurioAction = ActionDef;

impl CurioAction {
    pub fn unapply(
        &self,
        node: &mut Node,
        curio_key: usize,
        _target_pt: Point,
        metadata: &Metadata,
    ) -> Result<()> {
        let deleted_sprites = metadata.get_or_default(keys::DELETED_SPRITES)?;
        let damages = metadata.get_or_default(keys::DAMAGES)?;
        for (key, spr) in deleted_sprites.into_iter() {
            let head = damages
                .iter()
                .find(|dropped_square| dropped_square.0 == key)
                .map(|dropped_square| Ok(dropped_square.1))
                .unwrap_or_else(|| {
                    "Unable to restore deleted sprite, no squares were dropped from it?"
                        .fail_critical()
                })?;
            unsafe {
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

        if let Some((old_max_size, _)) = metadata.get(keys::MAX_SIZE_CHANGE)? {
            let target_id = metadata.expect(keys::TARGET_CURIO)?;
            node.with_curio_mut(target_id, |mut curio| {
                curio.set_max_size(old_max_size);
            })
            .ok_or_else(|| {
                format!("Unable to fix max size for missing curio {}", curio_key)
                    .fail_critical_msg()
            })?;
        }
        Ok(())
    }
    pub fn apply(&self, node: &mut Node, curio_key: usize, target_pt: Point) -> Result<Metadata> {
        if let Some(_target_type) = self
            .targets()
            .iter()
            .find(|target| target.matches(node, curio_key, target_pt))
        {
            if self
                .conditions()
                .iter()
                .all(|condition| condition.met(node, curio_key, target_pt))
            {
                let mut metadata = Metadata::new();
                metadata.put_optional(
                    keys::TARGET_CURIO,
                    node.with_curio_at(target_pt, |target| target.key()),
                )?;

                match self.effect() {
                    ActionEffect::DealDamage(dmg) => {
                        let (key, (dropped_pts, deleted_curio)) = node
                            .with_curio_at_mut(target_pt, |target| {
                                (target.key(), target.take_damage(*dmg))
                            })
                            .ok_or_else(|| {
                                "Invalid target for damage: ActionTarget must be a curio"
                                    .fail_reversible_msg()
                            })?;
                        metadata.put(keys::DAMAGES, &dropped_pts)?;
                        if let Some(deleted_curio) = deleted_curio {
                            let mut deleted_sprites =
                                metadata.get_or_default(keys::DELETED_SPRITES)?;
                            deleted_sprites.push((key, deleted_curio));
                            metadata.put(keys::DELETED_SPRITES, &deleted_sprites)?;
                        }
                    },
                    ActionEffect::IncreaseMaxSize { amount, bound } => {
                        let max_size_change: (usize, usize) = node
                            .with_curio_at_mut(target_pt, |mut target| {
                                let old_max = target.max_size();
                                let new_max = target.increase_max_size(*amount, *bound);
                                (old_max, new_max)
                            })
                            .ok_or_else(|| {
                                "Invalid target for increase size: ActionTarget must be a curio"
                                    .fail_reversible_msg()
                            })?;
                        metadata.put(keys::MAX_SIZE_CHANGE, &max_size_change)?;
                    },
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
        self.targets().contains(&ActionTarget::Enemy)
    }
}

impl ActionCondition {
    fn met(&self, node: &Node, curio_key: usize, target_pt: Point) -> bool {
        match self {
            ActionCondition::Size(range) => range.contains(&node.sprite_len(curio_key)),
            ActionCondition::TargetSize(range) => node
                .with_curio_at(target_pt, |target| range.contains(&target.size()))
                .unwrap_or(false),
            ActionCondition::TargetMaxSize(range) => node
                .with_curio_at(target_pt, |target| range.contains(&target.max_size()))
                .unwrap_or(false),
            _ => unimplemented!("TODO implement other conditions"),
        }
    }
}

impl ActionTarget {
    fn matches(&self, node: &Node, curio_key: usize, target_pt: Point) -> bool {
        match self {
            Self::Enemy => {
                let source_team = node.with_curio(curio_key, |curio| curio.team());
                let target_team = node.with_curio_at(target_pt, |curio| curio.team());
                source_team.is_some() && target_team.is_some() && source_team != target_team
            },
            Self::Ally => {
                // TODO shouldn't be able to target self.
                let source_team = node.with_curio(curio_key, |curio| curio.team());
                let target_team = node.with_curio_at(target_pt, |curio| curio.team());
                source_team.is_some() && target_team.is_some() && source_team == target_team
            },
            _ => unimplemented!("ActionTarget {:?} not implemented yet", self),
        }
    }
}
