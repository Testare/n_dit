use std::fs::File;
use std::time::Duration;

use bevy::prelude::*;
use bevy::scene::ScenePlugin;
use cq_term::demo::DemoNodeId;
use game_core::node::NodeId;
use simplelog::{LevelFilter, WriteLogger};

fn main() {
    setup_logging();
    let mut demo_node_id = DemoNodeId(None);
    for arg in std::env::args() {
        if arg == "n0" {
            demo_node_id.0 = Some(NodeId::new("node:demo", 0));
        } else if arg == "n1" {
            demo_node_id.0 = Some(NodeId::new("node:tutorial", 0));
        } else if arg == "n2" {
            demo_node_id.0 = Some(NodeId::new("node:tutorial", 1));
        } else if arg == "n3" {
            demo_node_id.0 = Some(NodeId::new("node:tutorial", 2));
        }
    }

    App::new()
        .insert_resource(demo_node_id)
        .add_plugins((
            AssetPlugin { ..default() },
            HierarchyPlugin,
            bevy::audio::AudioPlugin::default(),
            bevy::core::TaskPoolPlugin::default(),
            ScenePlugin,
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
