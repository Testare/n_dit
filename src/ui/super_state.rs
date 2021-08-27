use super::super::{Direction, GameState, Node, Point};
use super::DrawConfiguration;

pub struct SuperState {
    pub game: GameState,
    draw_config: DrawConfiguration,
    terminal_size: (usize, usize),
    selected_square: Point,
    selected_action_index: Option<usize>,
}

impl SuperState {
    pub fn from(node: Option<Node>) -> Self {
        // TODO This should be more safe, probably not an actual trait for UiState
        let (t_width, t_height) =
            crossterm::terminal::size().expect("Problem getting terminal size");

        SuperState {
            game: GameState::from(node),
            selected_square: (0, 0),
            selected_action_index: None,
            draw_config: DrawConfiguration::default(),
            terminal_size: (t_width.into(), t_height.into()),
        }
    }

    pub fn draw_config(&self) -> &DrawConfiguration {
        &self.draw_config
    }

    // TODO use Bounds
    pub fn terminal_size(&self) -> (usize, usize) {
        self.terminal_size
    }

    pub fn set_terminal_size(&mut self, bounds: (usize, usize)) {
        self.terminal_size = bounds;
    }

    pub fn selected_square(&self) -> Point {
        self.selected_square
    }

    pub fn selected_action_index(&self) -> Option<usize> {
        self.selected_action_index
    }

    pub fn move_selected_square(&mut self, direction: Direction, speed: usize) {
        self.selected_square = direction.add_to_point(
            self.selected_square,
            speed,
            self.game
                .node()
                .expect("TODO Why is this method called when there is no node?")
                .bounds(),
        )
    }
}
