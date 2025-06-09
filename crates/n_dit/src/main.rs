use std::borrow::Cow;
use std::path::PathBuf;
use std::time::Duration;

use bevy::diagnostic::FrameCount;
use bevy::prelude::*;
use bevy::remote::http::RemoteHttpPlugin;
use bevy::remote::RemotePlugin;
use bevy::scene::ScenePlugin;
use bevy::{app::RunMode, diagnostic::FrameCountPlugin};
use clap::Parser;
use cq_term::demo::UseDemoShader;
use flexi_logger::{FileSpec, LogSpecification, Logger};
use game_core::prelude::logging::std_log_fmt;
use game_core::prelude::Log;
use game_core::saving::CurrentSaveFile;

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
    save_file: Option<PathBuf>,

    /// Specify a frame to exit the program.
    /// If specified, kills the progrma after the specified frame.
    #[arg(long, value_name = "FRAME_NUM")]
    kill_frame: Option<u32>,
}

impl Plugin for CqCliPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(self.clone());
        app.insert_resource(UseDemoShader(self.demo_shader.unwrap_or(0)));
        if self.debug {
            app.add_plugins((RemotePlugin::default(), RemoteHttpPlugin::default()));
        }
        if self.save_file.is_some() {
            app.add_systems(
                Startup,
                move |mut save_file: ResMut<CurrentSaveFile>, cli_args: Res<CqCliPlugin>| {
                    **save_file = Cow::Owned(cli_args.save_file.clone().unwrap());
                },
            );
        }
        if self.kill_frame.is_some() {
            app.add_systems(PostUpdate, sys_kill_frame);
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
    setup_logging(&cq_cli, App::new())
        .register_type::<Name>()
        .add_plugins((
            cq_cli,
            AssetPlugin { ..default() },
            bevy::audio::AudioPlugin::default(),
            TaskPoolPlugin::default(),
            ScenePlugin,
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
fn setup_logging(cq_cli: &CqCliPlugin, mut app: App) -> App {
    if !cq_cli.debug {
        return app;
    }
    let file = if cq_cli.connect.is_some() {
        "debug.connect"
    } else {
        "debug"
    };
    let log_spec_str = if cq_cli.trace {
        "bevy_app::app=trace, cq_term=trace, game_core=trace, charmi=trace"
    } else {
        "bevy_app::app=debug, cq_term=debug, game_core=debug, charmi=debug"
    };
    let log = Log(Logger::with(LogSpecification::parse(log_spec_str).unwrap())
        .log_to_file(
            FileSpec::default()
                .basename(file)
                .o_directory::<String>(None)
                .suppress_timestamp(),
        )
        .format(std_log_fmt)
        .start()
        .unwrap());
    app.insert_resource(log);
    app
}

fn sys_kill_frame(
    res_frame_count: Res<FrameCount>,
    res_cq_cli: Res<CqCliPlugin>,
    mut evw_exit: EventWriter<AppExit>,
) {
    if res_frame_count.0 >= res_cq_cli.kill_frame.unwrap() {
        evw_exit.write(AppExit::Success);
    }
}
