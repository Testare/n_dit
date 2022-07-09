use game_core::{Bounds, Direction, GameCommand, GameState, Point};
use getset::{CopyGetters, Getters};

use super::{ClickTarget, DrawConfiguration, Layout, NodeUiState, UserInput};
// TODO Might be best to represent soem of this state as an enum state machine
#[derive(Debug, Getters, CopyGetters)]
pub struct SuperState {
    layout: Layout,
    draw_config: DrawConfiguration,
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
    pub fn from(state: &GameState) -> Self {
        // TODO This should be more safe, probably not an actual trait for UiState
        let (t_width, t_height) =
            crossterm::terminal::size().expect("Problem getting terminal size");

        SuperState {
            node_ui: state.node().map(NodeUiState::from),
            world_ui: WorldUiState::new(),
            layout: Layout::new((t_width, t_height).into()),
            draw_config: DrawConfiguration::default(),
            view: UiView::Node,
        }
    }

    pub fn action_for_char_pt(
        &self,
        pt: Point,
        state: &GameState,
        alt: bool,
        in_animation: bool,
    ) -> Vec<UiAction> {
        let ct = self.layout.click_target(self, state, pt);
        log::info!("Click at point [{:?}] -> CT [{:?}]", pt, ct);
        let ui_actions = match ct {
            Some(ClickTarget::Node(node_ct)) => {
                self.node_ui().unwrap().ui_actions_for_click_target(
                    state
                        .node()
                        .expect("Node click target whe nnode is not present"),
                    node_ct,
                    alt,
                )
            }
            _ => Vec::default(),
        };
        ui_actions
            .into_iter()
            .filter(|ui_action| !in_animation || *ui_action == UiAction::Quit)
            .collect()
    }

    pub fn draw_config(&self) -> &DrawConfiguration {
        &self.draw_config
    }

    // TODO remove from SuperState when Layout can handle it by itself
    pub fn terminal_size(&self) -> Bounds {
        // self.terminal_size
        self.layout.terminal_size()
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

    pub fn render(&self, state: &GameState) -> std::io::Result<bool> {
        self.layout.render(self, state)
    }

    pub fn ui_actions_for_input(&self, state: &GameState, user_input: UserInput) -> Vec<UiAction> {
        // TODO Perhaps have a method "is_animation_safe" property to indicate UI actions that can
        // apply even during animations
        let in_animation = state.animation().is_some();
        match user_input {
            UserInput::Quit => vec![UiAction::quit()], // Might be able to just return None here
            UserInput::Debug => panic!("Debug state: {:?}", self),
            UserInput::Resize(bounds) => vec![UiAction::set_terminal_size(bounds)],
            UserInput::Click(pt) => self.action_for_char_pt(pt, state, false, in_animation),
            UserInput::AltClick(pt) => self.action_for_char_pt(pt, state, true, in_animation),
            UserInput::Next => {
                if in_animation {
                    Vec::new()
                } else {
                    vec![UiAction::next()]
                }
            }
            _ => {
                if !in_animation {
                    self.node_ui
                        .as_ref()
                        .zip(state.node())
                        .map(|(node_ui, node)| node_ui.ui_action_for_input(node, user_input))
                        .unwrap_or_else(UiAction::none)
                } else {
                    Vec::new()
                }
            }
        }
    }

    pub fn apply_action(&mut self, ui_action: UiAction, state: &GameState) -> Result<(), String> {
        log::info!("Performing UiAction {:?}", ui_action);

        let SuperState {
            node_ui, layout, ..
        } = self;

        match &ui_action {
            UiAction::SetTerminalSize(bounds) => {
                layout.resize(*bounds);
            }
            UiAction::Quit => {
                panic!("Thanks for playing")
            }
            _ => {}
        }

        node_ui
            .as_mut()
            .zip(state.node())
            .map(|(node_ui, node)| node_ui.apply_action(node, &ui_action))
            .unwrap_or_else(|| Err("Node UI action, but no node".to_string()))?;

        layout.apply_action(&ui_action, node_ui.as_ref());
        Ok(())
    }
}

// TODO idea: UiAction is a public struct with a hidden "type" variable, this enum becomes
// UiActionType?
#[derive(PartialEq, Eq, Clone, Debug)]
pub enum UiAction {
    ChangeSelection,
    ConfirmSelection(Option<usize>),
    #[deprecated]
    ChangeSelectedMenuItem(Direction),
    MoveSelectedSquare {
        direction: Direction,
        speed: usize,
    }, // Do we really need this too when we have "SetSelectedSquare"?
    SetSelectedSquare(Point),
    SetSelectedMenuItem(usize),
    GameCommand(GameCommand),
    SetTerminalSize(Bounds),
    Quit,
}

type UiActions = Vec<UiAction>;

impl UiAction {
    pub fn ready_to_play() -> UiAction {
        UiAction::GameCommand(GameCommand::NodeReadyToPlay)
    }

    pub fn play_card(card_name: &str, target_access_point: Point) -> UiAction {
        UiAction::GameCommand(GameCommand::NodePlayCard {
            card_name: card_name.to_string(),
            target_access_point,
        })
    }

    pub fn perform_curio_action(action_name: &str, target: Point) -> UiAction {
        UiAction::GameCommand(GameCommand::NodeTakeAction {
            action_name: action_name.to_string(),
            target,
        })
    }

    pub fn activate_curio(curio_id: usize) -> UiAction {
        UiAction::GameCommand(GameCommand::NodeActivateCurio { curio_id })
    }

    pub fn deactivate_curio() -> UiAction {
        UiAction::GameCommand(GameCommand::NodeDeactivateCurio)
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
        UiAction::GameCommand(GameCommand::Next)
    }

    pub fn quit() -> UiAction {
        UiAction::Quit
    }

    pub fn undo() -> UiAction {
        UiAction::GameCommand(GameCommand::Undo)
    }

    pub fn is_quit(&self) -> bool {
        matches!(self, UiAction::Quit)
    }

    pub fn move_active_curio(dir: Direction) -> UiAction {
        UiAction::GameCommand(GameCommand::NodeMoveActiveCurio(dir))
    }

    #[deprecated]
    pub fn change_selected_menu_item(dir: Direction) -> UiAction {
        UiAction::ChangeSelectedMenuItem(dir)
    }

    pub fn set_selected_menu_item(index: usize) -> UiAction {
        UiAction::SetSelectedMenuItem(index)
    }

    pub fn confirm_selection(index: Option<usize>) -> UiAction {
        UiAction::ConfirmSelection(index)
    }

    pub fn change_selection() -> UiAction {
        UiAction::ChangeSelection
    }

    pub fn none() -> UiActions {
        Vec::new()
    }
}
