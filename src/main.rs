use std::fs::File;
use std::time::Duration;

use bevy::app::RunMode;
use bevy::prelude::*;
use bevy::scene::ScenePlugin;
use clap::Parser;
use cq_term::demo::{DemoNodeId, UseDemoShader};
use game_core::node::NodeId;
use simplelog::{LevelFilter, WriteLogger};

#[derive(Parser)]
#[command(author, version, about)]
struct CqCliPlugin {
    /// Select a demo node to load. Currently 0-3 supported
    #[arg(short, long, value_name = "NODE #")]
    node: Option<u8>,
    /// Activates logging and debuging to local file.
    #[arg(short, long)]
    debug: bool,
    /// Increases debug logging to next leve
    #[arg(short, long)]
    trace: bool,
    /// Specifies a server to connect to. Not currently implemented
    #[arg(short, long, value_name = "SERVER ADDRESS")]
    connect: Option<String>,
    /// Applies "demo shader" affect, a sliding rainbow
    #[arg(long = "rainbow", value_name = "RAINBOW HEIGHT")]
    demo_shader: Option<u32>,
    /// Runs game without a frame
    #[arg(short, long = "uncapped")]
    uncapped_fps: bool,
}

impl Plugin for CqCliPlugin {
    fn build(&self, app: &mut App) {
        let demo_node_id = DemoNodeId(self.node.and_then(|node_num| match node_num {
            0 => Some(NodeId::new("node:demo", 0)),
            1 => Some(NodeId::new("node:tutorial", 0)),
            2 => Some(NodeId::new("node:area1", 0)),
            3 => Some(NodeId::new("node:area1", 1)),
            _ => None,
        }));
        app.insert_resource(UseDemoShader(self.demo_shader.unwrap_or(0)));
        app.insert_resource(demo_node_id);
    }
}

fn main() {
    let cq_cli = CqCliPlugin::parse();

    let schedule_runner = if cq_cli.uncapped_fps {
        bevy::app::ScheduleRunnerPlugin {
            run_mode: RunMode::Loop { wait: None },
        }
    } else {
        bevy::app::ScheduleRunnerPlugin::run_loop(Duration::from_millis(25))
    };
    setup_logging(&cq_cli);
    App::new()
        .add_plugins((
            cq_cli,
            AssetPlugin { ..default() },
            HierarchyPlugin,
            bevy::audio::AudioPlugin::default(),
            bevy::core::TaskPoolPlugin::default(),
            ScenePlugin,
            TypeRegistrationPlugin,
            bevy::time::TimePlugin,
            schedule_runner,
            FrameCountPlugin,
            game_core::NDitCorePlugin,
            cq_term::CharmiePlugin,
            cq_term::demo::DemoPlugin,
        ))
        .run();
}

// Can set up more advanced CLI support in the future with clap
fn setup_logging(cq_cli: &CqCliPlugin) {
    if cq_cli.debug {
        let file = if cq_cli.connect.is_some() {
            "debug.connect.log"
        } else {
            "debug.log"
        };
        let log_level: LevelFilter = if cq_cli.trace {
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
