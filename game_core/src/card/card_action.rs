use crate::prelude::*;

#[derive(Clone, Component, Debug, Deref, FromReflect, Reflect)]
pub struct Actions(pub Vec<Entity>);

#[derive(Clone, Component, Debug, FromReflect, Reflect)]
pub struct Action {
    pub name: String,
}

#[derive(Clone, Component, Debug, Deref, FromReflect, Reflect)]
pub struct ActionRange(pub u32);

#[derive(Clone, Component, Debug, FromReflect, Reflect)]
pub enum ActionEffect {
    Damage(usize),
    Heal(usize),
}

#[derive(Clone, Component, Debug, Deref, DerefMut, FromReflect, Reflect)]
pub struct Prereqs(pub Vec<Prerequisite>);

#[derive(Clone, Debug, FromReflect, Reflect)]
pub enum Prerequisite {
    MinSize(u32),
}
