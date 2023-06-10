use std::fs::File;
use std::io::stdout;
use std::panic;
use std::time::Duration;

use bevy::app::ScheduleRunnerSettings;
use bevy::prelude::*;
use crossterm::{self, execute};
use old_game_core::{
    error, loader, node_from_def, AuthorityGameMaster, GameCommand, GameState, Inventory,
    NetworkGameMaster, NodeDef, Pickup,
};
use simplelog::{LevelFilter, WriteLogger};

fn main() -> error::Result<()> {
    setup_logging();

    let state = load_state();
    App::new()
        // .add_plugins(MinimalPlugins)
        .add_plugin(HierarchyPlugin)
        .add_plugin(bevy::core::TaskPoolPlugin::default())
        .add_plugin(TypeRegistrationPlugin)
        .add_plugin(bevy::time::TimePlugin)
        .insert_resource(ScheduleRunnerSettings::run_loop(Duration::from_millis(25)))
        .add_plugin(bevy::app::ScheduleRunnerPlugin)
        .add_plugin(FrameCountPlugin)
        .add_plugin(game_core::NDitCorePlugin)
        .add_plugin(n_dit::term::CharmiePlugin)
        .add_plugin(n_dit::demo::DemoPlugin)
        .run();

    Ok(())
}

fn load_state() -> GameState {
    let config = loader::Configuration {
        assets_folder: "./assets".to_string(),
    };
    let node_def = &loader::load_asset_dictionary::<NodeDef>(&config).unwrap()["Node0"];
    let curio_dict = loader::load_asset_dictionary(&config).unwrap();
    let action_dict = loader::load_asset_dictionary(&config).unwrap();
    let node = node_from_def(node_def, debug_inventory(), curio_dict, action_dict).unwrap();
    GameState::from(Some(node))
}

fn debug_inventory() -> Inventory {
    let mut inventory = Inventory::default();
    inventory.pick_up(Pickup::Card("Hack".to_string()));
    inventory.pick_up(Pickup::Card("Hack".to_string()));
    inventory.pick_up(Pickup::Card("Hack 3.0".to_string()));
    inventory.pick_up(Pickup::Card("Andy".to_string()));
    inventory.pick_up(Pickup::Card("Slingshot".to_string()));
    inventory
}

// Can set up more advanced CLI support in the future with clap
fn setup_logging() {
    if std::env::args().any(|arg| arg == "--debug") {
        let file = if std::env::args().any(|arg| arg == "--connect") {
            "debug.connect.log"
        } else {
            "debug.log"
        };
        // Should I do something in the future to make this append style instead of recreate file?
        WriteLogger::init(
            LevelFilter::Debug,
            simplelog::Config::default(),
            File::create(file).unwrap(),
        )
        .unwrap()
    }
}
