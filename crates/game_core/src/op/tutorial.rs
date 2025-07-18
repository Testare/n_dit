use bevy::ecs::entity::EntityHashSet;

use crate::{
    op::{Op, OpError, OpErrorUtils},
    prelude::*,
};

// TODO potential move this module out of op, despite its strong reliance on op, op doesn't really rely on it.
#[derive(Debug)]
pub struct TutorialPlugin;

impl Plugin for TutorialPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<InTutorial>()
            .register_type::<Tutorial>();
    }
}

#[derive(Component, Debug)]
pub struct TutorialState(Vec<String>);

#[derive(Component, Debug, Reflect)]
#[relationship(relationship_target=Tutorial)]
pub struct InTutorial(Entity);

#[derive(Component, Debug, Reflect)]
#[relationship_target(relationship=InTutorial)]
pub struct Tutorial(EntityHashSet);

pub trait TutorialOp: Op + Sized {
    fn can_perform_during_tutorial(
        &self,
        tutorial_state: &[String],
        advanced_check: impl FnOnce(&Self, &[String]) -> bool,
    ) -> bool;

    fn check_tutorial_simple(
        &self,
        q_tutorials: &Query<&TutorialState>,
        in_tutorial: Option<&InTutorial>,
    ) -> Result<(), OpError> {
        self.check_tutorial_advanced(q_tutorials, in_tutorial, |_, _| {
            log::error!("Tutorial op check used simple check, but the TutorialOp invoked the advanced check");
            false
        })
    }

    fn check_tutorial_advanced(
        &self,
        q_tutorials: &Query<&TutorialState>,
        in_tutorial: Option<&InTutorial>,
        advanced_check: impl FnOnce(&Self, &[String]) -> bool,
    ) -> Result<(), OpError> {
        if let Some(expected_tutorial_op) =
            in_tutorial.and_then(|&InTutorial(tutorial_id)| q_tutorials.get(tutorial_id).ok())
        {
            if !self.can_perform_during_tutorial(&expected_tutorial_op.0, advanced_check) {
                return Err("Not the action the tutorial indicated".invalid());
            }
        }
        Ok(())
    }
}
