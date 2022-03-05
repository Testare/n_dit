use super::super::{Bounds, GameState, Point};
use super::{SuperState, UiAction, UiView};

mod node_layout;

use node_layout::{NodeLayout, StandardNodeLayout};

// TODO Figure out Layout vs Render module boundaries
// TODO DrawConfiguration determines NodeLayout used?

trait SubLayout {
    unsafe fn render(&self, state: &SuperState) -> std::io::Result<bool>;
    fn scroll_to_pt(&mut self, pt: Point);
    unsafe fn action_for_char_pt(&self, state: &SuperState, pt: Point) -> Option<UiAction>;
    fn resize(&mut self, terminal_size: Bounds) -> bool;
    // fn can_be_rendered
    // fn update_size
    // TODO scroll(Direction)
}

// TODO Reevaluate if we want this to be a super layout, or break layout down for different uis, and the role of interplay between layout and ui, especially
// when there is a node
/// Represents all layout stuff. Its fields should be the configuration for preferred layout for in-node and out-of-node
/// Eventually, we want all layout implementation details to be obscured by this one struct
#[derive(Clone, Copy, Debug)]
pub struct Layout {
    node_layout: NodeLayout,
}

impl Layout {
    pub fn node_layout(&self) -> &NodeLayout {
        &self.node_layout
    }

    pub fn new(terminal_size: Bounds) -> Self {
        Layout {
            node_layout: NodeLayout::Standard(StandardNodeLayout::new(terminal_size, None, None)),
        }
    }

    pub fn scroll_node_to_pt(&mut self, pt: Point) {
        self.node_layout.scroll_to_pt(pt);
    }

    pub fn scroll_to_pt(&mut self, game_state: &GameState, pt: Point) {
        if game_state.node().is_some() {
            self.scroll_node_to_pt(pt);
        } // TODO World UI scorlling
    }

    pub fn render(&self, state: &SuperState) -> std::io::Result<bool> {
        if state.view() == UiView::Node {
            unsafe {
                // Only unsafe because it requires node to be present, but node IS present
                self.node_layout.render(state)
            }
        } else {
            unimplemented!("TODO World map not implemented")
        }
    }

    pub fn action_for_char_pt(&self, state: &SuperState, pt: Point) -> Option<UiAction> {
        if state.game.node().is_some() {
            // It should be safe, we have verified a node exists
            unsafe { self.node_layout.action_for_char_pt(state, pt) }
        } else {
            unimplemented!("TODO World map not implemented")
        }
    }

    pub fn resize(&mut self, bounds: Bounds) {
        self.node_layout.resize(bounds);
    }
}

// Will likely be used later when I figure out how to handle multiple layouts.
mod too_small_layout {
    use super::SubLayout;
    use crate::{Bounds, Point, SuperState, UiAction};
    use crossterm::execute;
    use std::io::stdout;

    #[derive(Clone, Copy, Debug)]
    pub struct TooSmallLayout(pub Bounds);

    impl SubLayout for TooSmallLayout {
        unsafe fn render(&self, _state: &SuperState) -> std::io::Result<bool> {
            let (available_width, available_height) = self.0.into();
            for i in 0..available_height {
                let blinds = if i % 2 == 0 { ">" } else { "<" };
                execute!(
                    stdout(),
                    crossterm::style::Print(blinds.repeat(available_width)),
                    crossterm::style::Print("\n".to_string()),
                    crossterm::cursor::MoveToColumn(0)
                )?;
            }
            Ok(false)
        }

        fn scroll_to_pt(&mut self, _pt: Point) {}
        unsafe fn action_for_char_pt(&self, _state: &SuperState, _pt: Point) -> Option<UiAction> {
            None
        }

        fn resize(&mut self, terminal_size: Bounds) -> bool {
            self.0 = terminal_size;
            true
        }
    }
}
