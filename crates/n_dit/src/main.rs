use std::fs::File;
use std::time::Duration;
use std::borrow::Cow;
use std::path::PathBuf;

use bevy::app::RunMode;
use bevy::prelude::*;
use bevy::scene::ScenePlugin;
use clap::Parser;
use cq_term::demo::{UseDemoShader};
use game_core::saving::CurrentSaveFile;
use simplelog::{LevelFilter, WriteLogger};

// NOCOMMIT delete this
#[derive(Clone, Parser, Resource)]
#[command(author, version, about)]
struct CqCliPlugin {
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
    /// Runs game without a frame cap
    #[arg(short, long = "uncapped")]
    uncapped_fps: bool,
    /// Specify save file to read from and write to.
    ///
    /// If no parent is specified, like `save.json`, the save file will be loaded from the default save directory.
    ///
    /// If a parent is specified, like `./save.json` or `/path/to/file/save.json`, the path will be
    /// resolved as you would expected.
    #[arg(short = 'f', long, value_name = "SAVE_FILE")]
    save_file: Option<PathBuf>
}

impl Plugin for CqCliPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(self.clone());
        app.insert_resource(UseDemoShader(self.demo_shader.unwrap_or(0)));
        if self.save_file.is_some() {
            app.add_systems(Startup, move |mut save_file: ResMut<CurrentSaveFile>, cli_args: Res<CqCliPlugin>| {
                **save_file = Cow::Owned(cli_args.save_file.clone().unwrap());
            });
        }
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
