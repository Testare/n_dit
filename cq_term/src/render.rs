use std::io::{stdout, Write};
use std::ops::Deref;
use std::time::{Duration, Instant};

use charmi::CharacterMapImage;
use game_core::NDitCoreSet;
use itertools::{EitherOrBoth, Itertools};

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
    render_cache: Vec<String>,
    rendering: CharacterMapImage,
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
    pub fn new(rendering: Vec<String>) -> Self {
        TerminalRendering {
            rendering: rendering.clone().into(),
            render_cache: rendering,
        }
    }

    pub fn update_if_changed(
        mut rendering: Mut<TerminalRendering>,
        new_rendering: CharacterMapImage,
    ) {
        if *rendering.deref().charmie() != new_rendering {
            rendering.update_charmie(new_rendering);
        }
    }

    pub fn update_charmie(&mut self, new_rendering: CharacterMapImage) {
        self.render_cache = (&self.rendering).into();
        self.rendering = new_rendering;
    }

    pub fn update(&mut self, new_rendering: Vec<String>) {
        self.rendering = new_rendering.into();
        self.render_cache = (&self.rendering).into();
    }

    fn update_from(&mut self, tr: &TerminalRendering) {
        self.rendering = tr.rendering.clone();
        self.render_cache.clone_from(&tr.render_cache);
    }

    pub fn string_rendering(&self) -> &[String] {
        &self.render_cache
    }

    pub fn charmie(&self) -> &CharacterMapImage {
        &self.rendering
    }

    pub fn clear(&mut self) {
        self.rendering = CharacterMapImage::new();
        self.render_cache = Vec::new();
    }
}

impl From<CharacterMapImage> for TerminalRendering {
    fn from(rendering: CharacterMapImage) -> Self {
        let render_cache = (&rendering).into();
        Self {
            rendering,
            render_cache,
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
    mut render_cache: Local<TerminalRendering>,
    mut render_pause: ResMut<RenderPause>,
) {
    // Clear cache on resize
    if let RenderPause(Some(pause_render_until)) = *render_pause {
        let now = Instant::now();
        if pause_render_until > now {
            return; // Do not render
        } else {
            render_cache.clear();
            crossterm::queue!(
                stdout(),
                crossterm::terminal::Clear(crossterm::terminal::ClearType::All)
            )
            .unwrap();
            **render_pause = None;
        }
    }
    if let Some(tr) = window.render_target.and_then(|id| renderings.get(id).ok()) {
        if *render_cache == *tr {
            return;
        }

        let render_result = render_with_cache(
            &Into::<Vec<String>>::into(&tr.rendering)[..],
            &Into::<Vec<String>>::into(&render_cache.rendering)[..],
            window.height(),
        );
        if let Result::Err(err) = render_result {
            log::error!("Error occurred in rendering: {:?}", err);
            return;
        }
        render_cache.update_from(tr);
    }
}

/// Helper method, does the actual rendering. If this is called, it is assumed
/// that the cache and rendering are not equal. The cached may be empty to just render
/// the whole thing
fn render_with_cache(
    rendering: &[String],
    cached: &[String],
    term_height: usize,
) -> std::io::Result<()> {
    let mut stdout = stdout();
    let rendering_height = rendering.len();
    for (line_num, line) in rendering.iter().zip_longest(cached.iter()).enumerate() {
        match line {
            EitherOrBoth::Both(line_to_render, cached_line) => {
                if line_to_render != cached_line {
                    log::trace!("Changed cache line, rendering: {}", line_num);
                    crossterm::queue!(
                        stdout,
                        crossterm::cursor::MoveTo(0, line_num as u16),
                        crossterm::style::Print(line_to_render.clone()),
                        crossterm::terminal::Clear(crossterm::terminal::ClearType::UntilNewLine)
                    )?;
                }
            },
            EitherOrBoth::Left(line_to_render) => {
                log::trace!("Rendering line without cache: {}", line_num);
                crossterm::queue!(
                    stdout,
                    crossterm::cursor::MoveTo(0, line_num as u16),
                    crossterm::style::Print(line_to_render.clone()),
                )?;
            },
            EitherOrBoth::Right(_cached_line) => {
                break;
            },
        }
    }
    if rendering_height < term_height {
        crossterm::queue!(
            stdout,
            crossterm::cursor::MoveTo(0, rendering_height as u16),
            crossterm::terminal::Clear(crossterm::terminal::ClearType::FromCursorDown)
        )?;
    }

    crossterm::queue!(stdout, crossterm::cursor::MoveTo(0, 0))?;
    stdout.flush()
}

impl PartialEq<TerminalRendering> for TerminalRendering {
    fn eq(&self, rhs: &TerminalRendering) -> bool {
        self.render_cache.iter().eq(rhs.render_cache.iter())
    }
}
