mod render_node;

use std::io::{stdout, Write};

use super::TerminalWindow;
use crate::term::node::NodeCursor;
use bevy::core::FrameCount;
use game_core::prelude::*;
use game_core::{self, EntityGrid, NodePiece, Team};
use itertools::{EitherOrBoth, Itertools};
use render_node::GlyphRegistry;

#[derive(Component, FromReflect, Reflect)]
pub struct TerminalRendering {
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

#[derive(Component, FromReflect, Reflect)]
pub struct CachedTerminalRendering {
    rendering: Vec<String>,
    last_update: u32,
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

pub struct RenderPlugin;

impl Plugin for RenderPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GlyphRegistry>()
            .add_system(render_node)
            .add_system(write_rendering_to_terminal);
    }
}

pub fn render_node(
    mut commands: Commands,
    windows: Query<&TerminalWindow>,
    mut node_grids: Query<
        (
            Entity,
            &EntityGrid,
            &NodeCursor,
            Option<&mut TerminalRendering>,
        ),
        With<game_core::Node>,
    >,
    node_pieces: Query<(&NodePiece, Option<&Team>)>,
    frame_count: Res<FrameCount>,
    glyph_registry: Res<GlyphRegistry>,
) {
    if let Some((entity, grid, node_cursor, rendering_opt)) = node_grids.iter_mut().next() {
        let grid_rendering =
            render_node::render_grid(windows, grid, node_cursor, node_pieces, &glyph_registry);
        if let Some(mut rendering) = rendering_opt {
            rendering.update(grid_rendering, frame_count.0);
        } else {
            let rendering = TerminalRendering::new(grid_rendering, frame_count.0);
            commands.get_entity(entity).unwrap().insert(rendering);
        }
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
        // Future: get different write streams per window?
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
            /*

            let mut stdout = stdout();
            // TODO logic to not render if not necessary
            crossterm::queue!(
                stdout,
                crossterm::cursor::MoveTo(0, 0),
                crossterm::terminal::Clear(crossterm::terminal::ClearType::FromCursorDown)
            );
            for line in tr.rendering().iter() {
                crossterm::queue!(
                    stdout,
                    crossterm::style::Print(line.clone()),
                    crossterm::style::Print("\n"),
                    crossterm::cursor::MoveToColumn(0),
                );
            }
            stdout.flush();*/
        }
    }
}

/// Helper method
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
