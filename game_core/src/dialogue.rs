use bevy_yarnspinner::events::PresentLineEvent;
use bevy_yarnspinner::prelude::*;

use crate::prelude::*;

#[derive(Debug)]
pub struct DialoguePlugin;

impl Plugin for DialoguePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(YarnSpinnerPlugin::new())
            .add_systems(Update, sys_dialogue_view.after(YarnSpinnerSystemSet));
    }
}

/*
pub enum DialogueType {
    ChatAlert, // Message appears in "chat box"
    Alert, // Pops up for a period of time. Is this really dialogue?
    Menu, // Interactive, but easily left
    Character, // Interactive, but cannot be easily left?
}
*/

#[derive(Component, Debug)]
pub struct DialogueOptions(Vec<LocalizedLine>);

#[derive(Component, Debug, Default)]
pub struct Dialogue(Option<LocalizedLine>);

pub fn sys_dialogue_view(
    mut evr_dialogue: EventReader<PresentLineEvent>,
    mut q_dialogue_runner: Query<Option<&mut Dialogue>, With<DialogueRunner>>,
) {
    for PresentLineEvent { line, source } in evr_dialogue.read() {
        if let Some(mut dialogue) = q_dialogue_runner.get_mut(*source).ok().flatten() {
            dialogue.0 = Some(line.clone());
        }
    }
}
