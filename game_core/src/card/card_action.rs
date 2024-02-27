use std::borrow::Borrow;

use bevy::ecs::system::QueryLens;
use getset::Getters;
use serde::{Deserialize, Serialize};

use super::{MaximumSize, MovementSpeed};
use crate::common::metadata::MetadataErr;
use crate::node::Curio;
// TODO figure out how to handle these Node imports to decrease coupling
use crate::prelude::*;

// Is this really the best place for this module?
// It is used by node-op, it is exported as part of the card module,
// but it is specific to card actions
pub mod key {
    use typed_key::{typed_key, Key};

    use super::*;

    pub const TARGET_POINT: Key<UVec2> = typed_key!("target");
    pub const DAMAGES: Key<Vec<UVec2>> = typed_key!("damages");
    pub const FATAL: Key<bool> = typed_key!("fatal");
    pub const TARGET_ENTITY: Key<Entity> = typed_key!("target_entity");
    pub const CLOSED_SQUARE: Key<bool> = typed_key!("closed");
    pub const OLD_SQUARE_STATUS: Key<bool> = typed_key!("old_square_status");
    pub const OLD_TARGET_CAPACITY: Key<u32> = typed_key!("old_capacity");
    pub const OLD_TARGET_MOVEMENT: Key<u32> = typed_key!("old_movement");
}

#[derive(Asset, Clone, Debug, Getters, Reflect)]
pub struct Action {
    pub(crate) range: Option<ActionRange>,
    pub(crate) id: String,
    #[getset(get = "pub")]
    pub(crate) effects: Vec<ActionEffect>,
    #[getset(get = "pub")]
    pub(crate) self_effects: Vec<ActionEffect>,
    #[getset(get = "pub")]
    pub(crate) target: ActionTarget,
    #[getset(get = "pub")]
    pub(crate) tags: Vec<String>,
    #[getset(get = "pub")]
    pub(crate) prereqs: Vec<Prerequisite>,
    pub(crate) description: String,
}

#[derive(Clone, Component, Debug, Default, Deref, Reflect)]
#[reflect(Component)]
pub struct Actions(pub Vec<Handle<Action>>);

#[derive(Copy, Clone, Component, Debug, Reflect)]
pub struct ActionRange {
    shape: RangeShape,
    max_range: u32,
    min_range: u32,
    headless: bool,
}

#[derive(Clone, Copy, Debug, Deserialize, Default, Reflect, Serialize)]
pub enum RangeShape {
    #[default]
    Diamond,
    Square,
    Circle,
}

#[derive(
    Clone, Copy, Component, Debug, Default, Deserialize, PartialEq, Eq, Reflect, Serialize,
)]
pub enum ActionTarget {
    Allies,
    ClosedSquare,
    Curios,
    #[default]
    Enemies,
    FreeSquare,
    None,
    Point,
}

#[derive(Clone, Component, Debug, Deserialize, Reflect, Serialize)]
#[non_exhaustive]
pub enum ActionEffect {
    Damage(usize),
    Heal(usize),
    Open,
    Close,
    ModifyMovement(i32),
    ModifyCapacity(i32),
    AddTag(String),
}

#[derive(Clone, Component, Debug, Deref, DerefMut, Reflect)]
pub struct Prereqs(pub Vec<Prerequisite>);

#[derive(Clone, Debug, Deserialize, Reflect, Serialize)]
pub enum Prerequisite {
    MinSize(u32),
    // TargetMaxSize (Limit how much you can expand a curio)
}

impl ActionEffect {
    pub fn valid_target(&self, target: &ActionTarget) -> bool {
        match self {
            Self::Open => ActionTarget::ClosedSquare == *target,
            Self::Close => ActionTarget::FreeSquare == *target,
            _ => matches!(
                target,
                ActionTarget::Allies | ActionTarget::Curios | ActionTarget::Enemies
            ),
        }
    }

    pub fn apply_effect(
        &self,
        grid: &mut Mut<EntityGrid>,
        _source: Entity,
        target: UVec2,
        entity_props: &mut Query<(AsDerefMut<MaximumSize>, AsDerefMut<MovementSpeed>), With<Curio>>,
    ) -> Result<Metadata, MetadataErr> {
        let mut action_metadata = Metadata::default();

        action_metadata.put(key::TARGET_POINT, target)?;
        Ok(match self {
            ActionEffect::Damage(dmg) => {
                if let Some(key) = grid.item_at(target) {
                    let damages = grid.list_back_n(key, *dmg);
                    action_metadata.put(key::TARGET_ENTITY, key)?;
                    action_metadata.put(key::DAMAGES, damages)?;
                    action_metadata.put(key::FATAL, grid.len_of(key) <= *dmg)?;
                    grid.pop_back_n(key, *dmg);
                }
                action_metadata
            },
            ActionEffect::Open => {
                if grid.square_is_closed(target) {
                    grid.open_square(target);
                    action_metadata.put(key::OLD_SQUARE_STATUS, false)?;
                }
                action_metadata
            },
            ActionEffect::Close => {
                if grid.square_is_free(target) {
                    grid.close_square(target);
                    action_metadata.put(key::OLD_SQUARE_STATUS, true)?;
                }
                action_metadata
            },
            ActionEffect::ModifyCapacity(capacity_change) => {
                if let Some(target_id) = grid.item_at(target) {
                    if let Ok((mut capacity, _)) = entity_props.get_mut(target_id) {
                        action_metadata.put(key::TARGET_ENTITY, target_id)?;
                        action_metadata.put(key::OLD_TARGET_CAPACITY, *capacity)?;
                        if *capacity_change > 0 {
                            *capacity = capacity.saturating_add(*capacity_change as u32);
                        } else {
                            *capacity = capacity.saturating_sub((-capacity_change) as u32);
                        }
                    }
                }
                action_metadata
            },
            ActionEffect::ModifyMovement(movement_change) => {
                if let Some(target_id) = grid.item_at(target) {
                    if let Ok((_, mut movement_speed)) = entity_props.get_mut(target_id) {
                        action_metadata.put(key::TARGET_ENTITY, target_id)?;
                        action_metadata.put(key::OLD_TARGET_MOVEMENT, *movement_speed)?;
                        if *movement_change > 0 {
                            *movement_speed =
                                movement_speed.saturating_add(*movement_change as u32);
                        } else {
                            *movement_speed =
                                movement_speed.saturating_sub((-movement_change) as u32);
                        }
                    }
                }
                action_metadata
            },
            ActionEffect::Heal(_healing) => todo!("Healing not implemented yet"),
            _ => todo!("Not implemented yet"),
        })
    }

    pub fn revert_effects(
        metadata: Metadata,
        grid: &mut Mut<EntityGrid>,
        mut curio_props: QueryLens<(AsDerefMut<MaximumSize>, AsDerefMut<MovementSpeed>)>,
    ) -> Result<(), MetadataErr> {
        let target_pt = metadata.get_required(key::TARGET_POINT)?;
        let mut curio_props = curio_props.query();
        if let Some(mut damages) = metadata.get_optional(key::DAMAGES)? {
            let target_entity = metadata.get_required(key::TARGET_ENTITY)?;
            let was_fatal = metadata.get_required(key::FATAL)?;
            if was_fatal {
                let head = damages
                    .pop()
                    .expect("attack could not have been fatal if no damage was dealt");
                grid.put_item(head, target_entity);
            }
            for pt in damages.into_iter().rev() {
                grid.push_back(pt, target_entity);
            }
        }
        if let Some(old_square_status) = metadata.get_optional(key::OLD_SQUARE_STATUS)? {
            if old_square_status {
                grid.open_square(target_pt);
            } else {
                grid.close_square(target_pt);
            }
        }
        if let Some(old_capacity) = metadata.get_optional(key::OLD_TARGET_CAPACITY)? {
            let target_entity = metadata.get_required(key::TARGET_ENTITY)?;
            if let Ok((mut max_size, _)) = curio_props.get_mut(target_entity) {
                *max_size = old_capacity;
            }
        }
        if let Some(old_movement) = metadata.get_optional(key::OLD_TARGET_MOVEMENT)? {
            let target_entity = metadata.get_required(key::TARGET_ENTITY)?;
            if let Ok((_, mut movement)) = curio_props.get_mut(target_entity) {
                *movement = old_movement;
            }
        }

        Ok(())
    }
}

impl ActionRange {
    pub fn new(range: u32) -> Self {
        ActionRange {
            max_range: range,
            min_range: 0,
            shape: RangeShape::Diamond,
            headless: false,
        }
    }

    pub fn shaped(mut self, shape: RangeShape) -> Self {
        self.shape = shape;
        self
    }

    pub fn headless(mut self, headless: bool) -> Self {
        self.headless = headless;
        self
    }

    pub fn is_headless(&self) -> bool {
        self.headless
    }

    pub fn max_range(&self) -> u32 {
        self.max_range
    }

    pub fn minimum_range(&self) -> u32 {
        self.min_range
    }

    pub fn shape(&self) -> RangeShape {
        self.shape
    }

    // TODO rename: with_min_range
    pub fn min_range(mut self, min_range: u32) -> Self {
        if min_range <= self.max_range {
            self.min_range = min_range;
        } else {
            log::error!(
                "Tried to set min range of an action below the max range: min {} > max {}",
                min_range,
                self.max_range
            );
        }
        self
    }

    pub fn in_range_of(&self, grid: &EntityGrid, source: Entity, target: UVec2) -> bool {
        if self.headless {
            grid.square_iter(source)
                .any(|sqr| self.in_range_of_pt(sqr.location(), target))
        } else {
            grid.head(source)
                .map(|head| self.in_range_of_pt(head, target))
                .unwrap_or_default()
        }
    }

    fn in_range_of_pt(&self, from_pt: UVec2, target: UVec2) -> bool {
        let dist = self.shape.dist(from_pt, target);
        self.min_range <= dist && dist <= self.max_range
    }

    pub fn in_range_of_pts<P: Borrow<UVec2>, I: IntoIterator<Item = P>>(
        &self,
        from_pts: I,
        target: UVec2,
    ) -> bool {
        from_pts
            .into_iter()
            .any(|pt| self.in_range_of_pt(*pt.borrow(), target))
    }
    pub fn pt_in_range<P: Borrow<UVec2>, I: IntoIterator<Item = P>>(
        &self,
        from_pts: I,
        target: UVec2,
    ) -> Option<UVec2> {
        from_pts.into_iter().find_map(|pt| {
            self.in_range_of_pt(*pt.borrow(), target)
                .then(|| *pt.borrow())
        })
    }
}

impl ActionTarget {
    pub fn valid_target<F: Fn(Entity) -> Option<Entity>>(
        &self,
        grid: &EntityGrid,
        source: Entity,
        target: UVec2,
        team_check: F,
    ) -> bool {
        match self {
            Self::None => true,
            Self::Curios => grid.item_at(target).and_then(team_check).is_some(),
            Self::Enemies => grid
                .item_at(target)
                .and_then(&team_check)
                .and_then(|target_team| Some(target_team != team_check(source)?))
                .unwrap_or(false),
            Self::Allies => {
                if let Some(target_entity) = grid.item_at(target) {
                    team_check(target_entity) == team_check(source)
                } else {
                    false
                }
            },
            Self::FreeSquare => grid.square_is_free(target),
            Self::ClosedSquare => grid.square_is_closed(target),
            Self::Point => true,
        }
    }
}

impl Prerequisite {
    pub fn satisfied(&self, grid: &Mut<EntityGrid>, source: Entity, _target: UVec2) -> bool {
        match self {
            Prerequisite::MinSize(min_size) => (*min_size as usize) <= grid.len_of(source),
        }
    }
}

impl RangeShape {
    fn dist(&self, from_pt: UVec2, target: UVec2) -> u32 {
        let dist = UVec2 {
            x: from_pt.x.abs_diff(target.x),
            y: from_pt.y.abs_diff(target.y),
        };
        match self {
            Self::Diamond => dist.x + dist.y,
            Self::Circle => dist.x * dist.x + dist.y * dist.y,
            Self::Square => dist.x.max(dist.y),
        }
    }
}
