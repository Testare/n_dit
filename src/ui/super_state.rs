use super::{DrawConfiguration, Layout, NodeUiState, UserInput};
use crate::{Bounds, Direction, GameAction, GameState, Node, Point, PointSet};

// TODO Might be best to represent soem of this state as an enum state machine
#[derive(Debug)]
pub struct SuperState {
    pub game: GameState,
    layout: Layout,
    draw_config: DrawConfiguration,
    terminal_size: (usize, usize),
    selection: Selection,
    node_ui: Option<NodeUiState>,
    world_ui: WorldUiState,
}

#[derive(Debug)]
pub struct WorldUiState {
    current_square: Point,
}

impl WorldUiState {
    fn new() -> Self {
        WorldUiState {
            current_square: (0, 0),
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
enum Selection {
    Grid,
    PauseMenu(Box<Selection>),
    SubMenu,
    SubMenu2,
    Node,
    World,
}

impl SuperState {
    pub fn from(node: Option<Node>) -> Self {
        // TODO This should be more safe, probably not an actual trait for UiState
        let (t_width, t_height) =
            crossterm::terminal::size().expect("Problem getting terminal size");

        SuperState {
            node_ui: node.as_ref().map(NodeUiState::from),
            world_ui: WorldUiState::new(),
            game: GameState::from(node),
            layout: Layout::new((t_width, t_height).into()),
            draw_config: DrawConfiguration::default(),
            terminal_size: (t_width.into(), t_height.into()),
            selection: Selection::Grid,
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
        // TODO use Layout, trigger recalculations, or use UiAction
        self.terminal_size = bounds;
    }

    pub fn selected_square(&self) -> Point {
        self.node_ui
            .as_ref()
            .map(|node| node.selected_square())
            .unwrap_or(self.world_ui.current_square)
    }

    pub fn selected_action_index(&self) -> Option<usize> {
        self.node_ui
            .as_ref()
            .and_then(|node_ui| node_ui.selected_action_index())
    }

    pub fn node_ui(&self) -> Option<&NodeUiState> {
        self.node_ui.as_ref()
    }

    pub fn render(&self) -> std::io::Result<bool> {
        self.layout.render(self)
    }

    pub fn game_state(&self) -> &GameState {
        &self.game
    }

    pub fn ui_action_for_input(&self, user_input: UserInput) -> Option<UiAction> {
        match user_input {
            UserInput::Quit => Some(UiAction::quit()), // Might be able to just return None here
            UserInput::Debug => panic!("Debug state: {:?}", self),
            UserInput::Resize(bounds) => Some(UiAction::set_terminal_size(bounds)),
            UserInput::Click(pt) => self.action_for_char_pt(pt),
            _ => self
                .node_ui
                .as_ref()
                .and_then(|node_ui| node_ui.ui_action_for_input(user_input)),
        }
    }

    pub fn apply_action(&mut self, ui_action: UiAction) -> Result<(), String> {
        if let UiAction::GameAction(game_action) = &ui_action {
            self.game.apply_action(game_action)?;
        }

        // TODO after ui action is refactored to properly separate game actions, move this below the match statement and remove clone
        let SuperState {
            game,
            node_ui,
            layout,
            ..
        } = self;

        node_ui
            .as_mut()
            .zip(game.node_mut())
            .map(|(node_ui, node)| node_ui.apply_action(node, layout, ui_action.clone()))
            .unwrap_or_else(|| Err("Node UI action, but no node".to_string()))?;

        match &ui_action {
            UiAction::SetTerminalSize(bounds) => {
                self.layout.resize(*bounds);
            }
            UiAction::Quit => {
                panic!("Thanks for playing")
            }
            _ => {}
        }
        Ok(())
    }
}

// TODO idea: UiAction is a public struct with a hidden "type" variable, this enum becomes
// UiActionType?
#[derive(Clone)]
pub enum UiAction {
    ChangeSelection,
    ConfirmSelection,
    ChangeSelectedMenuItem(Direction),
    MoveSelectedSquare { direction: Direction, speed: usize },
    SetSelectedSquare(Point),
    GameAction(GameAction),
    SetTerminalSize(Bounds),
    Quit,
}

impl UiAction {
    pub fn perform_sprite_action(action_index: usize, pnt: Point) -> UiAction {
        UiAction::GameAction(GameAction::take_sprite_action(action_index, pnt))
    }

    pub fn activate_sprite(sprite_key: usize) -> UiAction {
        UiAction::GameAction(GameAction::activate_sprite(sprite_key))
    }

    pub fn deactivate_sprite() -> UiAction {
        UiAction::GameAction(GameAction::deactivate_sprite())
    }

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
        UiAction::GameAction(GameAction::next())
    }

    pub fn quit() -> UiAction {
        UiAction::Quit
    }

    pub fn is_quit(&self) -> bool {
        matches!(self, UiAction::Quit)
    }

    pub fn move_active_sprite(dir: Direction) -> UiAction {
        UiAction::GameAction(GameAction::move_activee_sprite(vec![dir]))
    }

    pub fn change_selected_menu_item(dir: Direction) -> UiAction {
        UiAction::ChangeSelectedMenuItem(dir)
    }
}
