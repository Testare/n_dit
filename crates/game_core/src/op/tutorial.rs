use bevy::ecs::{entity::{EntityHashMap, EntityHashSet}, system::SystemId};
use bevy_yarnspinner::prelude::DialogueRunner;

use crate::{
    op::{Op, OpError, OpErrorUtils}, player::Player, prelude::*
};

// TODO potentially move this module out of op, despite its strong reliance on op, op doesn't really rely on it.
#[derive(Debug)]
pub struct TutorialPlugin;

impl Plugin for TutorialPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<InTutorial>()
            .register_type::<Tutorial>()
            .register_type::<TutorialState>()
            .register_type::<TutorialId>()
            .add_event::<TutorialIndication>()
            .add_systems(PreUpdate, sys_add_tutorial_commands_to_player)
            ;
    }
}

/// Marker component added after tutorial yarn commands are registered on a player's DialogueRunner.
/// Used to prevent re-registration and to avoid the `Added<DialogueRunner>` timing pitfall
/// (deferred command insertion means Added<T> fires too late — same tick as last_run).
#[derive(Component, Debug)]
struct TutorialCommandsAdded;

#[derive(Component, Debug, Reflect)]
#[type_path = "game_core::tutorial"]
#[reflect(Component)]
pub struct TutorialState(Vec<String>);

#[derive(Component, Debug, Reflect)]
#[relationship(relationship_target=Tutorial)]
#[type_path = "game_core::tutorial"]
#[reflect(Component)]
pub struct InTutorial(Entity);

#[derive(Component, Debug, Reflect)]
#[relationship_target(relationship=InTutorial)]
#[type_path = "game_core::tutorial"]
#[reflect(Component)]
pub struct Tutorial(EntityHashSet);

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
#[type_path = "game_core::tutorial"]
/// A named indicator tag scoped to a specific tutorial.
/// The second field is the [`Tutorial`] entity, not the player entity.
/// This allows scene files to reference a statically-known entity ID, and correctly
/// associates indicators with the tutorial rather than a specific player.
pub struct TutorialId(pub String, #[entities] pub Entity);

impl TutorialId {
    pub fn new<S: ToString>(name: S, tutorial_id: Entity) -> Self {
        Self(name.to_string(), tutorial_id)
    }
}

#[derive(Debug, Event)]
pub struct TutorialIndication(pub Entity);


pub trait TutorialOp: Op + Sized {
    fn can_perform_during_tutorial(
        &self,
        tutorial_state: &[String],
        advanced_check: impl FnOnce(&Self, &[String]) -> Result<bool, OpError>,
    ) -> Result<bool, OpError>;

    fn check_tutorial_simple(
        &self,
        q_tutorials: &Query<&TutorialState>,
        in_tutorial: Option<&InTutorial>,
    ) -> Result<(), OpError> {
        self.check_tutorial_advanced(q_tutorials, in_tutorial, |_, _| {
            log::error!("Tutorial op check used simple check, but the TutorialOp invoked the advanced check");
            Ok(false)
        })
    }

    fn check_tutorial_advanced(
        &self,
        q_tutorials: &Query<&TutorialState>,
        in_tutorial: Option<&InTutorial>,
        advanced_check: impl FnOnce(&Self, &[String]) -> Result<bool, OpError>,
    ) -> Result<(), OpError> {
        if let Some(expected_tutorial_op) =
            in_tutorial.and_then(|&InTutorial(tutorial_id)| q_tutorials.get(tutorial_id).ok())
        {
            if !self.can_perform_during_tutorial(&expected_tutorial_op.0, advanced_check)? {
                return Err("Not the action the tutorial indicated".invalid());
            }
        }
        Ok(())
    }
}

fn bind_tutorial_indicate(player_id: Entity, commands: &mut Commands) -> SystemId<In<String>, ()> {
    commands.register_system(move |
        In(tutorial_id): In<String>,
        mut evw_tutorial_indication: EventWriter<TutorialIndication>,
        q_tutorial_ids: Query<(&TutorialId, Entity)>,
        q_in_tutorial: Query<&InTutorial>,
    | {
        let Ok(&InTutorial(tutorial_entity)) = q_in_tutorial.get(player_id) else {
            log::error!("Failed TutorialIndication: player {player_id:?} is not in a tutorial");
            return;
        };
        if let Some((_, id)) = q_tutorial_ids
            .iter()
            .find(|(TutorialId(tid, tut_id), _)| tid == &tutorial_id && tut_id == &tutorial_entity)
        {
            evw_tutorial_indication.write(TutorialIndication(id));
            log::debug!("TutorialIndication sent for player {player_id:?}, for {tutorial_id:?} ({id:?})");
        } else {
            log::error!("Failed TutorialIndication: Unable to find {tutorial_id:?} for tutorial {tutorial_entity:?}");
        }
    })
}

fn sys_add_tutorial_commands_to_player(
    mut commands: Commands,
    mut tutorial_indicate_ids: Local<EntityHashMap<SystemId<In<String>, ()>>>,
    mut q_player_dr: Query<(Entity, &mut DialogueRunner), (With<Player>, Without<TutorialCommandsAdded>)>
) {
    // Could be a memory leak here if we make this massively multiplayer
    for (player_id, mut dr) in q_player_dr.iter_mut() {
        let tutorial_indicate_id = *tutorial_indicate_ids
            .entry(player_id)
            .or_insert_with(||bind_tutorial_indicate(player_id, &mut commands));

        dr.commands_mut().add_command("tutorial_indicate", tutorial_indicate_id);
        commands.entity(player_id).insert(TutorialCommandsAdded);
    }
}
