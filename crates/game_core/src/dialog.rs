use bevy::ecs::query::QueryData;
use bevy::ecs::schedule::common_conditions;
use bevy_yarnspinner::events::{
    DialogueCompleteEvent, ExecuteCommandEvent, NodeCompleteEvent, PresentLineEvent,
    PresentOptionsEvent,
};
use bevy_yarnspinner::prelude::*;
use getset::Getters;

use crate::op::{CoreOps, OpResult};
use crate::prelude::*;
use crate::shop::ShopOp;

#[derive(Debug, Event)]
pub struct DialogTrigger(pub Entity, pub String);

#[derive(Debug)]
pub struct DialogPlugin;

impl Plugin for DialogPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(YarnSpinnerPlugin::new())
            .add_event::<DialogTrigger>()
            .add_systems(
                PreUpdate,
                sys_setup_dialogue_runners.run_if(common_conditions::resource_added::<YarnProject>),
            )
            .add_systems(
                Update,
                (
                    (sys_dialog_view, sys_yarn_commands).chain(),
                    sys_dialog_response_to_node_op,
                )
                    .after(YarnSpinnerSystemSet),
            );
    }
}

// To help protect us from being too tightly coupled with bevy_yarnspinner
#[derive(Debug, QueryData)]
#[query_data(mutable)]
pub struct DialogInterface{
    dialog_runner: &'static mut DialogueRunner
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
    mut evr_dialogue_complete: EventReader<DialogueCompleteEvent>,
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
            dialog.options.clone_from(options);
        }
    }
    for DialogueCompleteEvent { source } in evr_dialogue_complete.read() {
        if let Ok((Some(mut dialog), _dialogue_runner)) = q_dialogue_runner.get_mut(*source) {
            dialog.line = None;
            dialog.options.clear();
        }
    }
}

fn sys_setup_dialogue_runners(
    mut commands: Commands,
    res_yarn: Res<YarnProject>,
    q_dialog_without_runner: Query<Entity, (With<Dialog>, Without<DialogueRunner>)>,
) {
    for id in q_dialog_without_runner.iter() {
        let dialogue_runner = res_yarn.create_dialogue_runner(&mut commands);
        commands.entity(id).insert(dialogue_runner);
    }
}

/// Node ops can create [DialogTrigger] events, which are used to move a dialog forward on a hidden
/// option.
fn sys_dialog_response_to_node_op(
    mut evr_triggers: EventReader<DialogTrigger>,
    mut q_dialog: Query<(&Dialog, &mut DialogueRunner)>,
) {
    for DialogTrigger(source, dialog_trigger_name) in evr_triggers.read() {
        let Ok((dialog, mut runner)) = q_dialog.get_mut(*source) else {
            continue;
        };
        let mut move_option: Option<OptionId> = None;
        for option in dialog.options.iter() {
            let Some(trigger) = option.line.attribute("trigger") else {
                continue;
            };
            if let Some(MarkupValue::String(trigger_name)) = trigger.property("name") {
                // TODO allow for more nuanced op reactions
                if trigger_name == dialog_trigger_name {
                    move_option = Some(option.id);
                    break;
                }
            }
        }
        if let Some(move_option) = move_option {
            if let Err(e) = runner.select_option(move_option) {
                log::error!(
                    " Error triggering option {move_option:?} from op path event {dialog_trigger_name:?}: {e:?}",
                );
            }
            // dialog
        }
    }
}

/// TODO make yarn commands more flexible
fn sys_yarn_commands(
    mut res_core_ops: ResMut<CoreOps>,
    mut evr_yarn_commands: EventReader<ExecuteCommandEvent>,
) {
    for ExecuteCommandEvent { command, source } in evr_yarn_commands.read() {
        match command.name.as_str() {
            "open_shop" => {
                if let Some(shop_sid_str) = command.parameters.first() {
                    match shop_sid_str.to_string().parse() {
                        Ok(shop_sid) => {
                            res_core_ops.request(*source, ShopOp::Enter(shop_sid));
                        },
                        Err(err) => {
                            log::error!("Error with dialog: unable to parse shop id [{shop_sid_str:?}]: {err:?}");
                        },
                    }
                } else {
                    log::error!("open_shop requires a parameter")
                }
            },
            _ => {},
        }
    }
}
