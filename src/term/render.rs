use std::io::{stdout, Write};

use super::TerminalWindow;
use game_core::prelude::*;
use itertools::{EitherOrBoth, Itertools};

#[derive(Component, FromReflect, Reflect)]
pub struct TerminalRendering {
    rendering: Vec<String>,
    last_update: u32,
}

#[derive(Component, FromReflect, Reflect)]
pub struct CachedTerminalRendering {
    rendering: Vec<String>,
    last_update: u32,
}

impl TerminalRendering {
    pub fn new(rendering: Vec<String>, last_update: u32) -> Self {
        TerminalRendering {
            rendering,
            last_update,
        }
    }

    pub fn update(&mut self, new_rendering: Vec<String>, frame_count: u32) {
        self.rendering = new_rendering;
        self.last_update = frame_count;
    }

    pub fn rendering(&self) -> &[String] {
        &self.rendering
    }
}

impl Default for TerminalRendering {
    fn default() -> Self {
        TerminalRendering {
            rendering: Vec::new(),
            last_update: 0,
        }
    }
}

impl CachedTerminalRendering {
    fn update_from(&mut self, tr: &TerminalRendering) {
        self.rendering = tr.rendering.clone();
        self.last_update = tr.last_update;
    }
}

impl From<&TerminalRendering> for CachedTerminalRendering {
    fn from(value: &TerminalRendering) -> Self {
        CachedTerminalRendering {
            rendering: value.rendering.clone(),
            last_update: value.last_update,
        }
    }
}

impl PartialEq<TerminalRendering> for CachedTerminalRendering {
    fn eq(&self, rhs: &TerminalRendering) -> bool {
        self.rendering.iter().eq(rhs.rendering.iter())
    }
}

impl PartialEq<CachedTerminalRendering> for TerminalRendering {
    fn eq(&self, rhs: &CachedTerminalRendering) -> bool {
        self.rendering.iter().eq(rhs.rendering.iter())
    }
}

#[derive(Default)]
pub struct RenderPlugin;

impl Plugin for RenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(write_rendering_to_terminal);
    }
}

pub fn write_rendering_to_terminal(
    mut commands: Commands,
    mut windows: Query<(
        Entity,
        &TerminalWindow,
        Option<&mut CachedTerminalRendering>,
    )>,
    renderings: Query<&TerminalRendering>,
) {
    for (window_entity, window, cached_rendering_opt) in windows.iter_mut() {
        if let Some(tr) = window.render_target.and_then(|id| renderings.get(id).ok()) {
            match cached_rendering_opt {
                Some(mut cached_rendering) => {
                    if cached_rendering.as_ref() == tr {
                        continue;
                    }
                    let render_result =
                        render_with_cache(&tr.rendering, &cached_rendering.rendering);
                    if let Result::Err(err) = render_result {
                        log::error!("Error occurred in rendering: {:?}", err);
                        continue;
                    }
                    cached_rendering.update_from(tr)
                }
                None => {
                    let render_result = render_with_cache(&tr.rendering, &[]);
                    if let Result::Err(err) = render_result {
                        log::error!("Error occurred in rendering (no-cache): {:?}", err);
                        continue;
                    }
                    commands
                        .entity(window_entity)
                        .insert(CachedTerminalRendering::from(tr));
                }
            }
        }
    }
}

/// Helper method, does the actual rendering. If this is called, it is assumed
/// that the cache and rendering are not equal. The cached may be empty to just render
/// the whole thing
fn render_with_cache(rendering: &[String], cached: &[String]) -> std::io::Result<()> {
    let mut stdout = stdout();
    for (line_num, line) in rendering.iter().zip_longest(cached.iter()).enumerate() {
        match line {
            EitherOrBoth::Both(line_to_render, cached_line) => {
                if line_to_render != cached_line {
                    log::debug!("Changed cache line, rendering: {}", line_num);
                    crossterm::queue!(
                        stdout,
                        crossterm::cursor::MoveTo(0, line_num as u16),
                        crossterm::style::Print(line_to_render.clone()),
                    )?;
                }
            }
            EitherOrBoth::Left(line_to_render) => {
                log::debug!("Rendering line without cache: {}", line_num);
                crossterm::queue!(
                    stdout,
                    crossterm::cursor::MoveTo(0, line_num as u16),
                    crossterm::style::Print(line_to_render.clone()),
                )?;
            }
            EitherOrBoth::Right(_cached_line) => {
                crossterm::queue!(
                    stdout,
                    crossterm::cursor::MoveTo(0, line_num as u16),
                    crossterm::terminal::Clear(crossterm::terminal::ClearType::FromCursorDown)
                )?;
            }
        }
    }

    crossterm::queue!(stdout, crossterm::cursor::MoveTo(0, 0))?;
    stdout.flush()
}
