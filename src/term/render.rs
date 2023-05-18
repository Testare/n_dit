mod render_node;

use std::io::{stdout, Write};

use bevy::core::FrameCount;
use game_core::{self, EntityGrid, NodePiece, Team};
use super::TerminalWindow;
use game_core::prelude::*;
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
    mut node_grids: Query<(Entity, &EntityGrid, Option<&mut TerminalRendering>), With<game_core::Node>>,
    node_pieces: Query<(&NodePiece, Option<&Team>)>,
    frame_count: Res<FrameCount>,
    glyph_registry: Res<GlyphRegistry>,
) {
    if let Some((entity, grid, rendering_opt)) = node_grids.iter_mut().next() {
        let grid_rendering = render_node::render_grid(windows, grid, node_pieces, &glyph_registry);
        if let Some(mut rendering) = rendering_opt {
            rendering.update(grid_rendering, frame_count.0);
        } else {
            let rendering = TerminalRendering::new(grid_rendering, frame_count.0);
            commands.get_entity(entity).unwrap().insert(rendering);
        }
    } 
}

pub fn write_rendering_to_terminal(
    windows: Query<&TerminalWindow>,
    mut node_grids: Query<&TerminalRendering>,
) {

    for window in windows.iter() {
        // Future: get different write streams per window?
        if let Some(tr) = window.render_target.and_then(|id|node_grids.get(id).ok()) {
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
            stdout.flush();
        }
    }

}