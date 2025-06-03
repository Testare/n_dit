use std::io::stdout;
use std::time::{Duration, Instant};

use charmi::CharmiImage;
use game_core::NDitCoreSet;

use super::TerminalWindow;
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

#[derive(Default)]
pub struct RenderTtyPlugin;

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
            .add_systems(
                RENDER_TTY_SCHEDULE,
                (apply_deferred, write_rendering_to_terminal)
                    .chain()
                    .in_set(RenderTtySet::RenderToTerminal),
            )
            .add_systems(PreUpdate, pause_rendering_on_resize)
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
            );
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
    renderings: Query<&TerminalRendering>,
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
            .charmi()
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
