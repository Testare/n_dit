use std::time::Instant;

use bevy::ecs::observer::Trigger;
use bevy::math::IVec3;
use bevy::render::gpu_readback::{Readback, ReadbackComplete};
use bevy::render::storage::ShaderStorageBuffer;
use charmi::{
    CharmiImage, CharmiImageSprite, CharmiRenderPlugin, MainView, TransformCh, ViewCh, ViewLayers,
};
use game_core::NDitCoreSet;

use super::TerminalWindow;
use crate::layout::{CalculatedSizeTty, GlobalTranslationTty, IsVisibleTty, VisibilityTty};
use crate::prelude::*;

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
pub struct RenderTtyPlugin;

#[derive(Clone, Component, Debug, Default, Deref, DerefMut)]
pub struct RenderCache(Option<CharmiImage>);

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
        app.init_resource::<RenderPause>()
            .configure_sets(
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
            )
            .add_plugins(CharmiRenderPlugin)
            .register_required_components::<TerminalRendering, CharmiImageSprite>()
            .register_required_components::<TerminalRendering, TransformCh>()
            .add_systems(
                RENDER_TTY_SCHEDULE,
                (
                    sys_update_charmi_sprites.in_set(RenderTtySet::RenderToTerminal),
                    sys_handle_resize,
                ),
            )
            .add_systems(Startup, sys_startup_render);
    }
}

pub fn sys_handle_resize(
    mut event_reader: EventReader<CrosstermEvent>,
    mut q_main_view_render_cache: Query<
        (&mut RenderCache, &mut TransformCh, &mut ViewCh),
        With<MainView>,
    >,
    mut ast_buffers: ResMut<Assets<ShaderStorageBuffer>>,
) {
    for event in event_reader.read() {
        let CrosstermEvent(crossterm::event::Event::Resize(width, height)) = event else {
            continue;
        };
        for (mut render_cache, mut transform, mut view) in q_main_view_render_cache.iter_mut() {
            **render_cache = None;
            // Do these changes propogate?
            transform.scale = UVec2 {
                x: *width as u32,
                y: *height as u32,
            };
            view.resize(transform.scale, ast_buffers.as_mut());
        }
    }
}

pub fn sys_startup_render(
    mut commands: Commands,
    res_window: Res<TerminalWindow>,
    mut ast_buffers: ResMut<Assets<ShaderStorageBuffer>>,
) {
    let main_view = ViewCh::new(100, *res_window.size(), ast_buffers.as_mut());
    let view_layers = [res_window
        .render_target()
        .expect("render target should always be some at this point (TODO ensure?)")]
    // TODO Confirmed that the above can sometimes panic, should fix this
    .into_iter()
    .collect();
    commands
        .spawn((
            MainView,
            ViewLayers(view_layers),
            Readback::buffer(main_view.buffer().clone()),
            main_view,
            TransformCh {
                scale: *res_window.size(),
                position: IVec3 { x: 0, y: 0, z: 0 },
            },
            RenderCache(None),
        ))
        .observe(
            |trigger: Trigger<ReadbackComplete>, mut render_cache: Query<&mut RenderCache>| {
                let Ok(mut last_image) = render_cache.get_mut(trigger.target()) else {
                    log::error!("Rendering with no render cache");
                    return;
                };
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

                // TODO BEFOREMERGE clear last image on resize
                // TODO instead of logging debug, perhaps save to a file?
                // log::trace!("Current screen render: {charmi:?}");
                if let Err(e) =
                    charmi.write_out_ansi(std::io::stdout(), last_image.deref().as_ref())
                {
                    log::error!("IO error when attempting to display view {e:?}");
                }
                **last_image = Some(charmi);
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
            &RenderOrder,
            IsVisibleTty,
        ),
        Or<(
            Changed<TerminalRendering>,
            Changed<CalculatedSizeTty>,
            Changed<GlobalTranslationTty>,
            Changed<RenderOrder>,
            Changed<VisibilityTty>,
        )>,
    >,
) {
    for (mut charmi_sprite, mut transform, tr, size, translation, render_order, is_visible) in
        q_terminal_renderings.iter_mut()
    {
        charmi_sprite.image = tr.charmi().clone();
        transform.position = translation.0.as_ivec2().extend(render_order.0 as i32);
        transform.scale = if is_visible { **size } else { UVec2::ZERO };
    }
}
