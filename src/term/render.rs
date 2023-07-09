use std::io::{stdout, Write};
use std::time::{Duration, Instant};

use bevy::core::FrameCount;
use bevy::ecs::system::{EntityCommand, EntityCommands};
use game_core::NDitCoreSet;
use itertools::{EitherOrBoth, Itertools};

use super::TerminalWindow;
use crate::charmie::CharacterMapImage;
use crate::term::prelude::*;

const PAUSE_RENDERING_ON_RESIZE_MILLIS: u64 = 500;

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub enum RenderTtySet {
    AdjustLayoutStyle,
    PreCalculateLayout,
    CalculateLayout,
    PostCalculateLayout,
    RenderLayouts,
    RenderToTerminal,
}

#[derive(Clone, Component)]
pub struct TerminalRendering {
    render_cache: Vec<String>,
    rendering: CharacterMapImage,
}

#[derive(Resource, Deref, DerefMut, Default)]
pub struct RenderPause(Option<Instant>);

#[derive(Default)]
pub struct RenderTtyPlugin;

impl TerminalRendering {
    pub fn new(rendering: Vec<String>) -> Self {
        TerminalRendering {
            rendering: rendering.clone().into(),
            render_cache: rendering,
        }
    }

    pub fn update_charmie(&mut self, new_rendering: CharacterMapImage) {
        self.render_cache = (&self.rendering).into();
        self.rendering = new_rendering.clone();
    }

    pub fn update(&mut self, new_rendering: Vec<String>) {
        self.rendering = new_rendering.clone().into();
        self.render_cache = (&self.rendering).into();
    }

    fn update_from(&mut self, tr: &TerminalRendering) {
        self.rendering = tr.rendering.clone();
        self.render_cache = tr.render_cache.clone();
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

impl Plugin for RenderTtyPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<RenderPause>()
            .add_systems(
                (apply_system_buffers, write_rendering_to_terminal)
                    .in_set(RenderTtySet::RenderToTerminal),
            )
            .add_system((pause_rendering_on_resize).in_base_set(CoreSet::PreUpdate))
            .configure_set(RenderTtySet::AdjustLayoutStyle.after(NDitCoreSet::ProcessCommandsFlush))
            .configure_set(RenderTtySet::AdjustLayoutStyle.before(RenderTtySet::CalculateLayout))
            .configure_set(RenderTtySet::PreCalculateLayout.before(RenderTtySet::CalculateLayout))
            .configure_set(RenderTtySet::CalculateLayout.before(RenderTtySet::PostCalculateLayout))
            .configure_set(RenderTtySet::PostCalculateLayout.before(RenderTtySet::RenderLayouts))
            .configure_set(RenderTtySet::RenderLayouts.before(RenderTtySet::RenderToTerminal));
    }
}

pub fn pause_rendering_on_resize(
    mut event_reader: EventReader<CrosstermEvent>,
    mut render_pause: ResMut<RenderPause>,
) {
    for event in event_reader.iter() {
        if matches!(event, CrosstermEvent::Resize { .. }) {
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

impl Default for TerminalRendering {
    fn default() -> Self {
        TerminalRendering {
            render_cache: Vec::new(),
            rendering: CharacterMapImage::default(),
        }
    }
}

impl PartialEq<TerminalRendering> for TerminalRendering {
    fn eq(&self, rhs: &TerminalRendering) -> bool {
        self.render_cache.iter().eq(rhs.render_cache.iter())
    }
}
