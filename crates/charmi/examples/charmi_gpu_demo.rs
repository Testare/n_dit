//! Simple example demonstrating the use of the [`Readback`] component to read back data from the GPU
//! using both a storage buffer and texture.

use std::fs::File;
use std::io::Write;
use std::sync::LazyLock;
use std::time::{Duration, Instant};

use bevy::app::ScheduleRunnerPlugin;
use bevy::diagnostic::DiagnosticsPlugin;
use bevy::log::LogPlugin;
use bevy::prelude::*;
use bevy::render::gpu_readback::{Readback, ReadbackComplete};
use bevy::render::render_resource::{AsBindGroup, ShaderRef};
use bevy::render::storage::ShaderStorageBuffer;
use bevy::time::common_conditions::on_timer;
use charmi::*;
use clap::Parser;

static TIMINGS_FILE: LazyLock<File> = LazyLock::new(|| {
    std::fs::File::options()
        .create(true)
        .append(true)
        .open("timings.log")
        .unwrap()
});

static LAZY_TEXT: LazyLock<Vec<u32>> = LazyLock::new(|| {
    "Have you ever heard the story of Darth  Plageius: The Wise? It is kinda a cool  story, but not that cool. I don't know, I mean, if I was a Jedi I would probablythink it wasn't cool enough to share, soyou probably wouldn't hear it from a    Jedi, but I still think it is pretty    cool. Anyways, it starts off with this  guy, and he's a bad guy. Sith! That's   right, they're the Sith. He is a Sith,  and he's got this Sith apprentice too.  He was really powerful. Like seriously. Like, he could even prevent people from dying, like the people he cared about.  You try and kill his wife? Bam, not     dead. His kids? Bam, not dead! That's   just hypothetical though, the story     doesn't actually say whether or not he  was married, or had kids. It does say hecares about somebody though, so maybe hewas! Anyways, he was so powerful, but   his apprentice killed him in his sleep. Pretty rude thing. And it turns out: He could prevent others from dying, but nothimself. So he dies! I think they call  that dramatic irony. Being able to      prevent other people from dying sounds  like it would be nice, but being able toprevent yourself from dying sounds much more useful.".chars().map(|ch|ch as u32).collect()
});

static LAZY_TEXT_2: LazyLock<Vec<u32>> = LazyLock::new(|| {
    "  Section A###### Section G#############                                        B Section C###### Section D#############B ##############                      ##                                          Section E#####  Section F##########     ##############  ###################    cool. Anyways, it starts off with this  guy, and he's a bad guy. Sith! That's   right, they're the Sith. He is a Sith,  and he's got this Sith apprentice too.  He was really powerful. Like seriously. Like, he could even prevent people from dying, like the people he cared about.  You try and kill his wife? Bam, not     dead. His kids? Bam, not dead! That's   just hypothetical though, the story     doesn't actually say whether or not he  was married, or had kids. It does say hecares about somebody though, so maybe hewas! Anyways, he was so powerful, but   his apprentice killed him in his sleep. Pretty rude thing. And it turns out: He could prevent others from dying, but nothimself. So he dies! I think they call  that dramatic irony. Being able to      prevent other people from dying sounds  like it would be nice, but being able toprevent yourself from dying sounds much more useful.".chars().map(|ch|ch as u32).collect()
});

/// Test using GPU for CLI/TUI graphics
#[derive(Clone, Copy, Default, Parser, Debug, Resource)]
#[command(version, about)]
struct TestArgs {
    /// Disable rendering, but output logs
    #[arg(long)]
    logs: bool,
    /// Not currently used
    #[arg(long)]
    debug: bool,
}

fn main() {
    let args = TestArgs::parse();
    let mut app = App::new();
    // TODO Figure out which plugins are required
    app.add_plugins((
        // PanicHandlerPlugin,
        TaskPoolPlugin::default(),
        TransformPlugin,
        DiagnosticsPlugin,
        ScheduleRunnerPlugin::default(),
    ));
    if args.logs {
        app.add_plugins(LogPlugin::default());
    }
    app.insert_resource(args)
        .add_plugins((CharmiRenderPlugin, TestExamplePlugin))
        .run();
}

// We need a plugin to organize all the systems and render node required for this example
struct TestExamplePlugin;
impl Plugin for TestExamplePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup)
            .add_systems(
                Update,
                sys_debug_movement.run_if(on_timer(Duration::from_millis(500))),
            )
            .add_plugins((
                MaterialChPlugin::<SegmentedUiMaterialCh>::default(),
                MaterialChPlugin::<RainbowTextBoxMaterialCh>::default(),
            ));
    }
}

#[derive(Component, Debug)]
pub struct DebugMovement;

fn sys_debug_movement(
    mut q_movement: Query<(&mut TransformCh, &mut CharmiImageSprite), With<DebugMovement>>,
) {
    for (mut transform, mut sprite) in q_movement.iter_mut() {
        transform.position.x += 1;
        // sprite.image.set_text(0, '#', "☐☐☐☐#☐☐☐☐");
        sprite.image = sprite
            .image
            .edit()
            .split_chars('<', '>')
            .gap_char('#')
            .add_line("☐そ☐")
            .add_line("☐##☐")
            .add_text("☐☐☐☐")
            .apply();
    }
}

fn setup(
    mut commands: Commands,
    mut ast_buffers: ResMut<Assets<ShaderStorageBuffer>>,
    mut mat: ResMut<Assets<SegmentedUiMaterialCh>>,
    mut mat_textbox: ResMut<Assets<RainbowTextBoxMaterialCh>>,
) {
    let text_buffer = ast_buffers.add(ShaderStorageBuffer::from((*LAZY_TEXT_2).clone()));
    log::info!("LAZY TEXT ->{LAZY_TEXT:?}");

    let segmented_ui = mat.add(SegmentedUiMaterialCh {
        pt1: UVec2 { x: 2, y: 2 },
        pt2: UVec2 { x: 17, y: 4 },
        pt3: UVec2 { x: 18, y: 5 },
        pt4: UVec2 { x: 38, y: 28 },
        // pt4: UVec2 { x: 220, y: 52 },
    });

    let text_box_asset = mat_textbox.add(RainbowTextBoxMaterialCh {
        text: "Not important right now".to_string(),
        text_buffer,
    });

    let image = CharmiImage::build_dynamic()
        .gap_char('#')
        .add_line(".--.")
        .add_line("-##-")
        .add_text(".--.")
        .build();

    let mini_unicode = CharmiImage::build_dynamic()
        .fg(4)
        .split_chars('<', '>')
        .add_text("バ")
        .build();

    let unicode_image = CharmiImage::build_dynamic()
        .fg(3)
        .bg(12)
        .split_chars('<', '>')
        .add_text("バカバカバカ")
        .build();

    let unicode_mask = CharmiImage::build_dynamic()
        .fg(2)
        .bg(13)
        .gap_char('#')
        .add_text("MAMA####DA")
        .build();

    commands.spawn((
        CharmiImageSprite { image },
        TransformCh {
            scale: UVec2 { x: 4, y: 3 },
            position: IVec3 { x: 5, y: 2, z: 10 },
        },
        DebugMovement,
    ));

    commands.spawn((
        CharmiImageSprite {
            image: unicode_image,
        },
        TransformCh {
            scale: UVec2 { x: 12, y: 1 },
            position: IVec3 { x: 3, y: 8, z: 10 },
        },
    ));
    commands.spawn((
        CharmiImageSprite {
            image: mini_unicode.clone(),
        },
        TransformCh {
            scale: UVec2 { x: 2, y: 1 },
            position: IVec3 { x: -1, y: 8, z: 10 },
        },
    ));

    commands.spawn((
        CharmiImageSprite {
            image: mini_unicode,
        },
        TransformCh {
            scale: UVec2 { x: 2, y: 1 },
            position: IVec3 { x: 41, y: 8, z: 10 },
        },
    ));

    commands.spawn((
        CharmiImageSprite {
            image: unicode_mask,
        },
        TransformCh {
            scale: UVec2 { x: 12, y: 1 },
            position: IVec3 { x: 4, y: 8, z: 20 },
        },
    ));
    commands.spawn((
        MaterialPaneCh(text_box_asset.clone()),
        TransformCh {
            scale: UVec2 { x: 3, y: 4 },
            position: IVec3 { x: 0, y: 2, z: 0 },
        },
    ));
    commands.spawn((
        MaterialPaneCh(text_box_asset.clone()),
        TransformCh {
            scale: UVec2 { x: 17, y: 3 },
            position: IVec3 { x: 2, y: 0, z: 0 },
        },
    ));
    commands.spawn((
        MaterialPaneCh(text_box_asset.clone()),
        TransformCh {
            scale: UVec2 { x: 17, y: 4 },
            position: IVec3 { x: 2, y: 2, z: 0 },
        },
    ));
    commands.spawn((
        MaterialPaneCh(text_box_asset.clone()),
        TransformCh {
            scale: UVec2 { x: 16, y: 26 },
            position: IVec3 { x: 2, y: 5, z: 0 },
        },
    ));

    commands.spawn((
        MaterialPaneCh(text_box_asset.clone()),
        TransformCh {
            scale: UVec2 { x: 25, y: 4 },
            position: IVec3 { x: 18, y: -1, z: 0 },
        },
    ));
    commands.spawn((
        MaterialPaneCh(text_box_asset.clone()),
        TransformCh {
            scale: UVec2 { x: 24, y: 4 },
            position: IVec3 { x: 18, y: 2, z: 0 },
        },
    ));
    commands.spawn((
        MaterialPaneCh(text_box_asset.clone()),
        TransformCh {
            scale: UVec2 { x: 18, y: 22 },
            position: IVec3 { x: 19, y: 6, z: 0 },
        },
    ));
    commands.spawn((
        MaterialPaneCh(segmented_ui),
        TransformCh {
            scale: UVec2 { x: 1, y: 1 },
            position: IVec3 { x: 42, y: 31, z: 1 },
        },
    ));

    let main_view_size = UVec2 {
        x: 42,
        // x: 119,
        // x: 239,
        // x: 478,
        y: 31,
        // y: 58,
        // y: 62,
        // y: 124,
    };
    let main_view = ViewCh::new(100, main_view_size, ast_buffers.as_mut());
    // Spawn the readback components. For each frame, the data will be read back from the GPU
    // asynchronously and trigger the `ReadbackComplete` event on this entity. Despawn the entity
    // to stop reading back the data.
    commands
        .spawn((
            MainView,
            Readback::buffer(main_view.buffer().clone()),
            main_view,
            TransformCh {
                scale: main_view_size,
                position: IVec3 { x: 0, y: 0, z: 0 },
            },
        ))
        .observe(
            |trigger: Trigger<ReadbackComplete>,
             mut last_time: Local<Option<Instant>>,
             mut last_image: Local<Option<CharmiImage>>,
             args: Res<TestArgs>| {
                if args.logs {
                    return;
                }
                /*
                 * ## MAJOR OPTIMIZATION IDEA
                 *
                 * Look into this: After rendering is complete, run a GPU task for each view
                 * This task compares the last rendered view to the current rendered view and
                 * helps determine which of the following strategies to use for rendering:
                 * 1. Skip outputing render (Images are identical)
                 * 2. Output changed charcells (Using ANSI codes to swap to coordinates)?
                 * 3. Output changed lines (Using ANSI codes to swap to line, but re-rendering
                 *    whole line)
                 * 4. Output whole rendering (Enough of the image has changed that there is no
                 *    point going line by line)
                 *
                 * The process should also probably copy the current rendering over the last
                 * rendering to cache it for the next comparison.
                 *
                 * For this to work, we would probably have a component on views that output to
                 * configure thresholds, identify which lines/characters need to be changed, and
                 * to store the buffer for the cached rendering.
                 *
                 * Be sure to take resizes into consideration as well.
                 *
                 */
                let time_diff = last_time
                    .as_ref()
                    .map(|last_time| last_time.elapsed().as_nanos())
                    .unwrap_or(0);
                *last_time = Some(Instant::now());

                let charmi: CharmiImage = CharmiImage::from(trigger.event());
                let start_time = Instant::now();

                if let Err(e) = charmi.write_out_ansi(std::io::stdout(), last_image.as_ref()) {
                    log::error!("IO error when attempting to display view {e:?}");
                }
                *last_image = Some(charmi);

                let time_to_buffer = start_time.elapsed().as_nanos();
                let _ = writeln!(&*TIMINGS_FILE, "{time_to_buffer},{time_diff}");
            },
        );
}

#[derive(Asset, Debug, Clone, AsBindGroup, TypePath)]
pub struct RainbowTextBoxMaterialCh {
    pub text: String,
    #[storage(0, visibility(compute))]
    pub text_buffer: Handle<ShaderStorageBuffer>,
}

impl MaterialCh for RainbowTextBoxMaterialCh {
    fn shader() -> ShaderRef {
        "shaders/textbox.wgsl".into()
    }
}

#[derive(Asset, Debug, Clone, AsBindGroup, TypePath)]
pub struct SegmentedUiMaterialCh {
    #[uniform(0)]
    pub pt1: UVec2,
    #[uniform(1)]
    pub pt2: UVec2,
    #[uniform(2)]
    pub pt3: UVec2,
    #[uniform(3)]
    pub pt4: UVec2,
}

impl MaterialCh for SegmentedUiMaterialCh {
    fn shader() -> ShaderRef {
        "shaders/segmented_ui.wgsl".into()
    }
}
