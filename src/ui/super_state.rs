use super::super::{Bounds, Direction, GameAction, GameState, Node, Point};
use super::{DrawConfiguration, Layout, UserInput};

#[derive(Debug)]
pub struct SuperState {
    pub game: GameState,
    layout: Layout,
    draw_config: DrawConfiguration,
    terminal_size: (usize, usize),
    selected_square: Point, // Might be a property of layout?
    selected_action_index: Option<usize>,
}

impl SuperState {
    pub fn from(node: Option<Node>) -> Self {
        // TODO This should be more safe, probably not an actual trait for UiState
        let (t_width, t_height) =
            crossterm::terminal::size().expect("Problem getting terminal size");

        SuperState {
            game: GameState::from(node),
            layout: Layout::new((t_width, t_height).into()),
            selected_square: (0, 0),
            selected_action_index: None,
            draw_config: DrawConfiguration::default(),
            terminal_size: (t_width.into(), t_height.into()),
        }
    }

    pub fn action_for_char_pt(&self, pt: Point) -> Option<UiAction> {
        self.layout.action_for_char_pt(self, pt)
    }

    pub fn draw_config(&self) -> &DrawConfiguration {
        &self.draw_config
    }

    // TODO remove from SuperState when Layout can handle it by itself
    pub fn terminal_size(&self) -> (usize, usize) {
        self.terminal_size
    }

    pub fn set_terminal_size(&mut self, bounds: (usize, usize)) {
        // TODO use Layout, trigger recalculations
        self.terminal_size = bounds;
    }

    pub fn selected_square(&self) -> Point {
        self.selected_square
    }

    pub fn selected_action_index(&self) -> Option<usize> {
        self.selected_action_index
    }

    pub fn render(&self) -> std::io::Result<bool> {
        self.layout.render(self)
    }

    pub fn set_selected_square(&mut self, pt: Point) {
        self.selected_square = pt
    }

    pub fn move_selected_square(&mut self, direction: Direction, speed: usize) {
        self.selected_square = direction.add_to_point(
            self.selected_square,
            speed,
            self.game
                .node()
                .expect("TODO Why is this method called when there is no node?")
                .bounds(),
        );
        let SuperState {
            layout,
            game,
            selected_square,
            ..
        } = self;
        layout.scroll_to_pt(game, *selected_square);
    }

    pub fn game_state(&self) -> &GameState {
        &self.game
    }

    pub fn ui_action_for_input(&self, user_input: UserInput) -> Option<UiAction> {
        match user_input {
            UserInput::Dir(dir) => Some(UiAction::move_selected_square(dir, 1)),
            UserInput::AltDir(dir) => Some(UiAction::move_selected_square(dir, 2)),
            UserInput::Quit => Some(UiAction::quit()), // Might be able to just return None here
            UserInput::Debug => panic!("Debug state: {:?}", self),
            UserInput::Resize(bounds) => Some(UiAction::set_terminal_size(bounds)),
            UserInput::Click(pt) => self.action_for_char_pt(pt),
            _ => None,
        }
    }

    pub fn apply_action(&mut self, ui_action: UiAction) -> Result<(), String> {
        match ui_action {
            UiAction::MoveSelectedSquare { direction, speed } => {
                self.move_selected_square(direction, speed);
                Ok(())
            }
            UiAction::SetSelectedSquare(pt) => {
                self.set_selected_square(pt);
                Ok(())
            }
            UiAction::DoGameAction(game_action) => self.game.apply_action(game_action),
            UiAction::SetTerminalSize(bounds) => {
                self.layout.resize(bounds);
                Ok(())
            }
            UiAction::Quit => {
                panic!("Thanks for playing")
            }
        }
    }
}

pub enum UiAction {
    MoveSelectedSquare { direction: Direction, speed: usize },
    SetSelectedSquare(Point),
    DoGameAction(GameAction),
    SetTerminalSize(Bounds),
    Quit,
}

impl UiAction {
    pub fn move_selected_square(direction: Direction, speed: usize) -> UiAction {
        UiAction::MoveSelectedSquare { direction, speed }
    }

    pub fn set_selected_square(pt: Point) -> UiAction {
        UiAction::SetSelectedSquare(pt)
    }

    pub fn set_terminal_size(bounds: Bounds) -> UiAction {
        UiAction::SetTerminalSize(bounds)
    }

    pub fn next() -> UiAction {
        UiAction::DoGameAction(GameAction::next())
    }

    pub fn quit() -> UiAction {
        UiAction::Quit
    }

    pub fn is_quit(&self) -> bool {
        if let UiAction::Quit = self {
            true
        } else {
            false
        }
    }
}
