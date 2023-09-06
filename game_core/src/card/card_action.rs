use std::borrow::Borrow;

use bitvec::macros::internal::funty::Floating;

use crate::common::metadata::MetadataErr;
use crate::prelude::*;

// Is this really the best place for this module?
// It is used by node-op, it is exported as part of the card module,
// but it is specific to card actions
pub mod key {
    use typed_key::{typed_key, Key};

    use super::*;

    pub const DAMAGES: Key<Vec<UVec2>> = typed_key!("damages");
    pub const FATAL: Key<bool> = typed_key!("fatal");
    pub const TARGET_ENTITY: Key<Entity> = typed_key!("target_entity");
}

#[derive(Clone, Component, Debug, Deref, Reflect)]
pub struct Actions(pub Vec<Entity>);

#[derive(Clone, Component, Debug, Reflect)]
pub struct Action {
    pub name: String,
}

#[derive(Copy, Clone, Component, Debug, Reflect)]
pub struct ActionRange {
    shape: RangeShape,
    max_range: u32,
    min_range: u32,
    headless: bool,
}

#[derive(Clone, Copy, Debug, Default, Reflect)]
pub enum RangeShape {
    #[default]
    Diamond,
    Square,
    Circle,
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

#[derive(Clone, Component, Copy, Debug, Reflect)]
pub enum ActionEffect {
    Damage(usize),
    Heal(usize),
}

#[derive(Clone, Component, Debug, Deref, DerefMut, Reflect)]
pub struct Prereqs(pub Vec<Prerequisite>);

#[derive(Clone, Debug, Reflect)]
pub enum Prerequisite {
    MinSize(u32),
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

impl Prerequisite {
    pub fn satisfied(&self, grid: &Mut<EntityGrid>, source: Entity, _target: UVec2) -> bool {
        match self {
            Prerequisite::MinSize(min_size) => (*min_size as usize) <= grid.len_of(source),
        }
    }
}

impl ActionEffect {
    pub fn apply_effect(
        &self,
        grid: &mut Mut<EntityGrid>,
        _source: Entity,
        target: UVec2,
    ) -> Result<Metadata, MetadataErr> {
        let mut action_metadata = Metadata::default();
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
            ActionEffect::Heal(_healing) => todo!("Healing not implemented yet"),
        })
    }
}
