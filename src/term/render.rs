use std::io::{stdout, Write};

use super::TerminalWindow;
use game_core::prelude::*;
use itertools::{EitherOrBoth, Itertools};

#[derive(Clone, Component, FromReflect, Reflect)]
pub struct TerminalRendering {
    rendering: Vec<String>,
    last_update: u32,
}

#[derive(Default)]
pub struct RenderPlugin;

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

    fn update_from(&mut self, tr: &TerminalRendering) {
        self.rendering = tr.rendering.clone();
        self.last_update = tr.last_update;
    }

    pub fn rendering(&self) -> &[String] {
        &self.rendering
    }
}

impl Plugin for RenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(write_rendering_to_terminal);
    }
}

pub fn write_rendering_to_terminal(
    window: Res<TerminalWindow>,
    renderings: Query<&TerminalRendering>,
    mut render_cache: Local<TerminalRendering>,
) {
    if let Some(tr) = window.render_target.and_then(|id| renderings.get(id).ok()) {
        if *render_cache == *tr {
            return;
        }

        let render_result = render_with_cache(&tr.rendering, &render_cache.rendering);
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

impl Default for TerminalRendering {
    fn default() -> Self {
        TerminalRendering {
            rendering: Vec::new(),
            last_update: 0,
        }
    }
}

impl PartialEq<TerminalRendering> for TerminalRendering {
    fn eq(&self, rhs: &TerminalRendering) -> bool {
        self.rendering.iter().eq(rhs.rendering.iter())
    }
}
