use crate::prelude::*;

#[derive(Clone, Component, Debug, Deref, FromReflect, Reflect)]
pub struct Actions(Vec<Action>);

#[derive(Clone, Debug, FromReflect, Reflect)]
pub struct Action {
    pub name: String,
    pub range: u32,
    pub effect: ActionEffect,
    pub description: String,
    pub prereqs: Vec<Prerequisite>,
    // tags
}

#[derive(Clone, Debug, FromReflect, Reflect)]
pub enum ActionEffect {
    Damage(usize),
    Heal(usize),
}

#[derive(Clone, Debug, FromReflect, Reflect)]
pub enum Prerequisite {
    MinSize(u32),
}

impl Actions {
    pub fn new(actions: Vec<Action>) -> Self {
        Actions(actions)
    }
}
