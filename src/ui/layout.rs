use game_core::{Bounds, GameState, Point};

use super::{ClickTarget, DrawConfiguration, NodeUiState, SuperState, UiAction, UiView};

mod node_layout;

use node_layout::{NodeLayout, StandardNodeLayout};

// TODO Figure out Layout vs Render module boundaries
// TODO DrawConfiguration determines NodeLayout used?

trait SubLayout {
    fn apply_action(&mut self, ui_action: &UiAction, node_ui: Option<&NodeUiState>);
    unsafe fn render(&self, state: &SuperState, game_state: &GameState) -> std::io::Result<bool>;
    fn scroll_to_pt(&mut self, pt: Point);
    fn resize(&mut self, terminal_size: Bounds) -> bool;
    fn click_target(
        &self,
        state: &SuperState,
        game_state: &GameState,
        pt: Point,
    ) -> Option<ClickTarget>;
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
    pub fn terminal_size(&self) -> Bounds {
        self.node_layout.terminal_size()
    }
    pub fn apply_action(&mut self, ui_action: &UiAction, node_ui: Option<&NodeUiState>) {
        self.node_layout.apply_action(ui_action, node_ui);
    }

    pub fn click_target(
        &self,
        state: &SuperState,
        game_state: &GameState,
        pt: Point,
    ) -> Option<ClickTarget> {
        if game_state.node().is_some() {
            self.node_layout.click_target(state, game_state, pt)
        } else {
            None
        }
    }

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

    pub fn render(&self, state: &SuperState, game_state: &GameState) -> std::io::Result<bool> {
        if state.view() == UiView::Node {
            unsafe {
                // Only unsafe because it requires node to be present, but node IS present
                self.node_layout.render(state, game_state)
            }
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
    use std::io::stdout;

    use crossterm::execute;
    use game_core::{Bounds, GameState, Point};

    use super::{ClickTarget, NodeUiState, SubLayout};
    use crate::{SuperState, UiAction};

    #[derive(Clone, Copy, Debug)]
    pub struct TooSmallLayout(pub Bounds);

    impl SubLayout for TooSmallLayout {
        unsafe fn render(
            &self,
            _state: &SuperState,
            _game_state: &GameState,
        ) -> std::io::Result<bool> {
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

        fn resize(&mut self, terminal_size: Bounds) -> bool {
            self.0 = terminal_size;
            true
        }

        fn click_target(&self, _: &SuperState, _: &GameState, _: Point) -> Option<ClickTarget> {
            None
        }

        fn apply_action(&mut self, _: &UiAction, _: Option<&NodeUiState>) {}
    }
}
