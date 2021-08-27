use super::configuration::{DrawConfiguration};
use super::game::{GameState, Node};
use super::{Direction, Point};
use std::num::NonZeroUsize;

pub mod layout;
mod render;


pub struct SuperState {
    pub ui: UiState,
    pub game: GameState,
}

impl SuperState {
    pub fn from(node: Option<Node>) -> Self {
        SuperState {
            game: GameState::from(node),
            ui: UiState::default(),
        }
    }
}

pub struct UiState {
    draw_config: DrawConfiguration,
    terminal_size: (usize, usize),
    selected_square: Point,
    selected_action_index: Option<usize>,
}

impl UiState {
    pub fn draw_config(&self) -> &DrawConfiguration {
        &self.draw_config
    }

    pub fn selected_square(&self) -> Point {
        self.selected_square
    }

    pub fn selected_action_index(&self) -> Option<usize> {
        self.selected_action_index
    }

    pub fn move_selected_square(&mut self, direction: Direction, node: &Node, speed: usize) {
        self.selected_square = direction.add_to_point(self.selected_square, speed, node.bounds())
    }
}

#[derive(Clone, Copy)]
pub struct Window {
    scroll_x: usize,
    scroll_y: usize,
    width: NonZeroUsize,
    height: NonZeroUsize,
}

impl Window {
    fn of(width: NonZeroUsize, height: NonZeroUsize) -> Self {
        Window {
            height,
            scroll_x: 0,
            scroll_y: 0,
            width,
        }
    }
}

impl Default for UiState {
    fn default() -> Self {
        // TODO This should be more safe, probably not an actual trait for UiState
        let (t_width, t_height) =
            crossterm::terminal::size().expect("Problem getting terminal size");
        UiState {
            selected_square: (0, 0),
            selected_action_index: None,
            draw_config: DrawConfiguration::default(),
            terminal_size: (t_width.into(), t_height.into()),
        }
    }
}

impl Default for Window {
    fn default() -> Self {
        Window {
            scroll_x: 0,
            scroll_y: 0,
            width: unsafe { NonZeroUsize::new_unchecked(usize::MAX) },
            height: unsafe { NonZeroUsize::new_unchecked(usize::MAX) },
        }
    }
}