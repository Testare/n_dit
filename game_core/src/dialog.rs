use bevy::ecs::schedule::common_conditions;
use bevy_yarnspinner::events::{
    ExecuteCommandEvent, NodeCompleteEvent, PresentLineEvent, PresentOptionsEvent,
};
use bevy_yarnspinner::prelude::*;
use getset::Getters;

use crate::prelude::*;
use crate::shop::{InShop, ShopId};

#[derive(Debug)]
pub struct DialogPlugin;

impl Plugin for DialogPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(YarnSpinnerPlugin::new())
            .add_systems(
                PreUpdate,
                sys_setup_dialogue_runners.run_if(common_conditions::resource_added::<YarnProject>),
            )
            .add_systems(
                Update,
                (sys_dialog_view, sys_yarn_commands)
                    .chain()
                    .after(YarnSpinnerSystemSet),
            );
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

fn sys_setup_dialogue_runners(
    mut commands: Commands,
    res_yarn: Res<YarnProject>,
    q_dialog_without_runner: Query<Entity, (With<Dialog>, Without<DialogueRunner>)>,
) {
    for id in q_dialog_without_runner.iter() {
        let dialogue_runner = res_yarn.create_dialogue_runner();
        commands.entity(id).insert(dialogue_runner);
    }
}

/// TODO make yarn commands more flexible
fn sys_yarn_commands(
    mut commands: Commands,
    mut evr_yarn_commands: EventReader<ExecuteCommandEvent>,
    q_shops: Query<(Entity, &ShopId)>,
) {
    for ExecuteCommandEvent { command, source } in evr_yarn_commands.read() {
        match command.name.as_str() {
            "open_shop" => {
                if let Some(shop_sid_str) = command.parameters.first() {
                    let shop_sid_str = shop_sid_str.to_string();
                    let shop_id = q_shops.iter().find_map(|(id, shop_sid)| {
                        (shop_sid_str == shop_sid.0.to_string()).then_some(id)
                    });
                    if let Some(shop_id) = shop_id {
                        commands.entity(*source).insert(InShop(shop_id));
                    } else {
                        log::error!("Could not find shop {shop_sid_str:?}")
                    }
                } else {
                    log::error!("open_shop requires a parameter")
                }
            },
            "jump" => {},
            _ => {
                log::error!("Unrecognized yarn command")
            },
        }
    }
}
