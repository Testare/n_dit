use std::io::stdout;
use std::time::{Duration, Instant};

use bevy::ecs::observer::Trigger;
use bevy::math::IVec3;
use bevy::render::gpu_readback::{Readback, ReadbackComplete};
use bevy::render::storage::ShaderStorageBuffer;
use charmi::{CharmiImage, CharmiImageSprite, CharmiRenderPlugin, MainView, TransformCh, ViewCh};
use game_core::NDitCoreSet;

use super::TerminalWindow;
use crate::layout::{CalculatedSizeTty, GlobalTranslationTty};
use crate::prelude::*;

const PAUSE_RENDERING_ON_RESIZE_MILLIS: u64 = 500;

pub const RENDER_TTY_SCHEDULE: Update = Update;

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub enum RenderTtySet {
    AdjustLayoutStyle,
    PreCalculateLayout,
    CalculateLayout,
    PostCalculateLayout, // TODO probably should rename "RenderElements"
    RenderLayouts,
    RenderToTerminal,
}

#[derive(Clone, Component, Debug, Default)]
pub struct TerminalRendering {
    rendering: CharmiImage,
}

#[derive(Resource, Deref, DerefMut, Default)]
pub struct RenderPause(Option<Instant>);

/// A component that hints at the what order entities are rendered in. Systems
/// (like layout) can generate these components to help input_events and other
/// systems know when components overlap, which is is on top.
///
/// **Mutating this field does NOT change the render order**, just makes other
/// systems think the render order is different. Unless you are making an
/// alternative to the [`crate::layout`] module, you probably should not be
/// mutating it
#[derive(Clone, Component, Copy, Debug, Default, Deref, Reflect)]
#[reflect(Component)]
pub struct RenderOrder(pub(crate) u32);

#[derive(Debug, Default)]
pub struct RenderTtyPlugin {
    pub alternate_rendering: bool,
}

impl TerminalRendering {
    pub fn new(rendering: CharmiImage) -> Self {
        TerminalRendering { rendering }
    }

    pub fn update_charmi(&mut self, new_rendering: CharmiImage) {
        self.rendering = new_rendering;
    }

    pub fn clear(&mut self) {
        self.rendering = CharmiImage::default();
    }

    pub(crate) fn charmi(&self) -> &CharmiImage {
        &self.rendering
    }
}

impl From<CharmiImage> for TerminalRendering {
    fn from(rendering: CharmiImage) -> Self {
        Self::from(&rendering)
    }
}

impl From<&CharmiImage> for TerminalRendering {
    fn from(rendering: &CharmiImage) -> Self {
        Self {
            rendering: rendering.clone(),
        }
    }
}

impl Plugin for RenderTtyPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<RenderPause>().configure_sets(
            RENDER_TTY_SCHEDULE,
            (
                NDitCoreSet::PostProcessUiOps,
                RenderTtySet::AdjustLayoutStyle,
                RenderTtySet::PreCalculateLayout,
                RenderTtySet::CalculateLayout,
                RenderTtySet::PostCalculateLayout,
                RenderTtySet::RenderLayouts,
                RenderTtySet::RenderToTerminal,
            )
                .chain(),
        );
        if self.alternate_rendering {
            app.add_plugins(CharmiRenderPlugin)
                // .register_required_components::<TerminalRendering, CharmiImageSprite>()
                .register_required_components::<TerminalRendering, TransformCh>()
                .add_systems(
                    RENDER_TTY_SCHEDULE,
                    sys_update_charmi_sprites.in_set(RenderTtySet::RenderToTerminal),
                )
                .add_systems(Startup, sys_startup_render);
        } else {
            app.add_systems(
                RENDER_TTY_SCHEDULE,
                (apply_deferred, write_rendering_to_terminal)
                    .chain()
                    .in_set(RenderTtySet::RenderToTerminal),
            )
            .add_systems(PreUpdate, pause_rendering_on_resize);
        }
    }
}

pub fn sys_startup_render(
    mut commands: Commands,
    res_window: Res<TerminalWindow>,
    mut ast_buffers: ResMut<Assets<ShaderStorageBuffer>>,
) {
    let main_view = ViewCh::new(100, *res_window.size(), ast_buffers.as_mut());
    commands
        .spawn((
            MainView,
            Readback::buffer(main_view.buffer().clone()),
            main_view,
            TransformCh {
                scale: *res_window.size(),
                position: IVec3 { x: 0, y: 0, z: 0 },
            },
        ))
        .observe(
            |trigger: Trigger<ReadbackComplete>, mut last_image: Local<Option<CharmiImage>>| {
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
                let charmi: CharmiImage = CharmiImage::from(trigger.event());

                if let Err(e) = charmi.write_out_ansi(std::io::stdout(), last_image.as_ref()) {
                    log::error!("IO error when attempting to display view {e:?}");
                }
                *last_image = Some(charmi);
            },
        );
    //
}

pub fn sys_update_charmi_sprites(
    mut q_terminal_renderings: Query<
        (
            &mut CharmiImageSprite,
            &mut TransformCh,
            &TerminalRendering,
            &CalculatedSizeTty,
            &GlobalTranslationTty,
        ),
        Or<(
            Changed<TerminalRendering>,
            Changed<CalculatedSizeTty>,
            Changed<GlobalTranslationTty>,
        )>,
    >,
) {
    for (mut charmi_sprite, mut transform, tr, size, translation) in
        q_terminal_renderings.iter_mut()
    {
        charmi_sprite.image = tr.charmi().clone();
        transform.position = translation.0.as_ivec2().extend(translation.1 as i32);
        transform.scale = **size;
    }
}

pub fn pause_rendering_on_resize(
    mut event_reader: EventReader<CrosstermEvent>,
    mut render_pause: ResMut<RenderPause>,
) {
    for event in event_reader.read() {
        if matches!(
            event,
            CrosstermEvent(crossterm::event::Event::Resize { .. })
        ) {
            **render_pause =
                Some(Instant::now() + Duration::from_millis(PAUSE_RENDERING_ON_RESIZE_MILLIS));
        }
    }
}

pub fn write_rendering_to_terminal(
    window: Res<TerminalWindow>,
    renderings: Query<&CharmiImageSprite>,
    mut render_cache: Local<Option<CharmiImage>>,
    mut render_pause: ResMut<RenderPause>,
) {
    // Clear cache on resize
    if let RenderPause(Some(pause_render_until)) = *render_pause {
        let now = Instant::now();
        if pause_render_until > now {
            return; // Do not render
        } else {
            *render_cache = None;
            crossterm::queue!(
                stdout(),
                crossterm::terminal::Clear(crossterm::terminal::ClearType::All)
            )
            .unwrap();
            **render_pause = None;
        }
    }
    if let Some(tr) = window.render_target.and_then(|id| renderings.get(id).ok()) {
        let tr = tr
            .image
            .resize(window.width() as u32, window.height() as u32, None);
        if render_cache.as_ref() == Some(&tr) {
            return;
        }

        let render_result = tr.write_out_ansi(stdout(), render_cache.as_ref());

        if let Result::Err(err) = render_result {
            log::error!("Error occurred in rendering: {:?}", err);
            return;
        }
        *render_cache = Some(tr.clone());
    }
}
