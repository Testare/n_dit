use std::fs::File;
use std::time::Duration;

use bevy::prelude::*;
use simplelog::{LevelFilter, WriteLogger};

fn main() {
    setup_logging();

    App::new()
        .add_plugins((
            AssetPlugin { ..default() },
            HierarchyPlugin,
            bevy::audio::AudioPlugin::default(),
            bevy::core::TaskPoolPlugin::default(),
            TypeRegistrationPlugin,
            bevy::time::TimePlugin,
            bevy::app::ScheduleRunnerPlugin::run_loop(Duration::from_millis(25)),
            FrameCountPlugin,
            game_core::NDitCorePlugin,
            cq_term::CharmiePlugin,
            cq_term::demo::DemoPlugin,
        ))
        .run();
}

// Can set up more advanced CLI support in the future with clap
fn setup_logging() {
    if std::env::args().any(|arg| arg == "--debug") {
        let file = if std::env::args().any(|arg| arg == "--connect") {
            "debug.connect.log"
        } else {
            "debug.log"
        };
        let log_level: LevelFilter = if std::env::args().any(|arg| arg == "--trace") {
            LevelFilter::Trace
        } else {
            LevelFilter::Debug
        };
        WriteLogger::init(
            log_level,
            simplelog::ConfigBuilder::new()
                .set_target_level(LevelFilter::Error)
                .build(),
            File::create(file).unwrap(),
        )
        .unwrap()
    }
}
