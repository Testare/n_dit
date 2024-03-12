use bevy_yarnspinner::events::{NodeCompleteEvent, PresentLineEvent, PresentOptionsEvent};
use bevy_yarnspinner::prelude::*;
use getset::Getters;

use crate::prelude::*;

#[derive(Debug)]
pub struct DialogPlugin;

impl Plugin for DialogPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(YarnSpinnerPlugin::new())
            .add_systems(Update, sys_dialog_view.after(YarnSpinnerSystemSet));
    }
}

/*
pub enum DialogType {
    ChatAlert, // Message appears in "chat box"
    Alert, // Pops up for a period of time. Is this really dialogue?
    Menu, // Interactive, but easily left
    Character, // Interactive, but cannot be easily left?
}
*/

#[derive(Component, Debug, Default, Getters)]
pub struct Dialog {
    #[getset(get = "pub")]
    line: Option<LocalizedLine>,
    #[getset(get = "pub")]
    next_line: Option<LocalizedLine>,
    #[getset(get = "pub")]
    options: Vec<DialogueOption>,
}

pub fn sys_dialog_view(
    mut evr_dialogue_line: EventReader<PresentLineEvent>,
    mut evr_dialogue_options: EventReader<PresentOptionsEvent>,
    mut evr_dialogue_node_complete: EventReader<NodeCompleteEvent>,
    mut q_dialogue_runner: Query<(Option<&mut Dialog>, &mut DialogueRunner)>,
) {
    let last_line = "lastline".to_string();
    for PresentLineEvent { line, source } in evr_dialogue_line.read() {
        if let Ok((Some(mut dialog), mut dialogue_runner)) = q_dialogue_runner.get_mut(*source) {
            dialog.options.clear();
            if line.metadata.contains(&last_line) {
                dialog.next_line = Some(line.clone());
                dialogue_runner.continue_in_next_update();
            } else {
                dialog.line = Some(line.clone());
            }
        }
    }
    for PresentOptionsEvent { options, source } in evr_dialogue_options.read() {
        if let Ok((Some(mut dialog), _dialogue_runner)) = q_dialogue_runner.get_mut(*source) {
            dialog.line = dialog.next_line.take();
            dialog.options = options.clone();
        }
    }
    for NodeCompleteEvent {
        node_name: _,
        source,
    } in evr_dialogue_node_complete.read()
    {
        if let Ok((Some(mut dialog), _dialogue_runner)) = q_dialogue_runner.get_mut(*source) {
            dialog.line = None;
            dialog.options.clear();
        }
    }
}
