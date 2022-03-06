use super::{DrawConfiguration, Layout, NodeUiState, UserInput};
use crate::{Bounds, Direction, GameAction, GameState, Node, Point};
use getset::{CopyGetters, Getters};

// TODO Might be best to represent soem of this state as an enum state machine
#[derive(Debug, Getters, CopyGetters)]
pub struct SuperState {
    pub game: GameState,
    layout: Layout,
    draw_config: DrawConfiguration,
    terminal_size: (usize, usize),
    #[get_copy = "pub"]
    view: UiView,
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiView {
    Node, // Do not set this view when node is none
    _NodePauseMenu,
    _World,
    _WorldPauseMenu,
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
            view: UiView::Node,
        }
    }

    pub fn action_for_char_pt(&self, pt: Point, in_animation: bool) -> Option<UiAction> {
        self.layout
            .action_for_char_pt(self, pt)
            .filter(|ui_action| !in_animation || *ui_action == UiAction::Quit)
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
        // TODO Perhaps have a method "is_animation_safe" property to indicate UI actions that can
        // apply even during animations
        let in_animation = self.game.animation().is_some();
        match user_input {
            UserInput::Quit => Some(UiAction::quit()), // Might be able to just return None here
            UserInput::Debug => panic!("Debug state: {:?}", self),
            UserInput::Resize(bounds) => Some(UiAction::set_terminal_size(bounds)),
            UserInput::Click(pt) => self.action_for_char_pt(pt, in_animation),
            UserInput::Next => Some(UiAction::next()).filter(|_| in_animation),
            _ => {
                if !in_animation {
                    self.node_ui
                        .as_ref()
                        .zip(self.game.node())
                        .and_then(|(node_ui, node)| node_ui.ui_action_for_input(node, user_input))
                } else {
                    None
                }
            }
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
#[derive(PartialEq, Eq, Clone, Debug)]
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
        UiAction::GameAction(GameAction::move_active_sprite(vec![dir]))
    }

    pub fn change_selected_menu_item(dir: Direction) -> UiAction {
        UiAction::ChangeSelectedMenuItem(dir)
    }

    pub fn confirm_selection() -> UiAction {
        UiAction::ConfirmSelection
    }

    pub fn change_selection() -> UiAction {
        UiAction::ChangeSelection
    }
}
