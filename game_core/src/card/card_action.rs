use crate::prelude::*;

#[derive(Clone, Component, Debug, Deref, Reflect)]
pub struct Actions(pub Vec<Entity>);

#[derive(Clone, Component, Debug, Reflect)]
pub struct Action {
    pub name: String,
}

#[derive(Clone, Component, Debug, Deref, Reflect)]
pub struct ActionRange(u32);

#[derive(Clone, Component, Debug, Reflect)]
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
        ActionRange(range)
    }

    pub fn in_range(&self, grid: &Mut<EntityGrid>, source: Entity, target: UVec2) -> bool {
        grid.head(source)
            .map(|head| head.manhattan_distance(&target) <= self.0)
            .unwrap_or_default()
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
    pub fn apply_effect(&self, grid: &mut Mut<EntityGrid>, _source: Entity, target: UVec2) {
        match self {
            ActionEffect::Damage(dmg) => {
                if let Some(key) = grid.item_at(target) {
                    grid.pop_back_n(key, *dmg);
                    // TODO pop_back_n returns removed locations and old head as result for other systems
                }
            },
            ActionEffect::Heal(_healing) => todo!("Healing not implemented yet"),
        }
    }
}
