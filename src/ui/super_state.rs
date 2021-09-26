use super::{DrawConfiguration, Layout, NodeUiState, UserInput};
use crate::{Bounds, Direction, GameAction, GameState, Node, Piece, Point, PointSet};

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

    // Use on nodeUi instead
    #[deprecated]
    pub fn selected_square(&self) -> Point {
        self.node_ui.as_ref().unwrap().selected_square()
    }

    pub fn selected_action_index(&self) -> Option<usize> {
        self.node_ui
            .as_ref()
            .and_then(|node_ui| node_ui.selected_action_index())
    }

    pub fn render(&self) -> std::io::Result<bool> {
        self.layout.render(self)
    }

    pub fn set_selected_square(&mut self, pt: Point) {
        self.node_ui.as_mut().unwrap().set_selected_square(pt);
    }

    fn set_default_selected_action(&mut self) {
        self.node_ui.as_mut().unwrap().set_default_selected_action();
    }

    pub fn move_selected_square(
        &mut self,
        direction: Direction,
        speed: usize,
        range_limit: Option<PointSet>,
    ) {
        let new_pt = direction.add_to_point(
            self.node_ui.as_ref().unwrap().selected_square(),
            speed,
            self.game
                .node()
                .expect("TODO Why is this method called when there is no node?")
                .bounds(),
        );
        if let Some(point_set) = range_limit {
            if !point_set.contains(new_pt) {
                return;
            }
        }
        self.set_selected_square(new_pt);
        let SuperState {
            layout,
            game,
            node_ui,
            ..
        } = self;
        layout.scroll_to_pt(game, node_ui.as_ref().unwrap().selected_square());
    }

    pub fn game_state(&self) -> &GameState {
        &self.game
    }

    pub fn ui_action_for_input(&self, user_input: UserInput) -> Option<UiAction> {
        match user_input {
            UserInput::Dir(dir) => {
                if self.selection == Selection::Grid {
                    if self.game.active_sprite_key().is_some()
                        && self.selected_action_index().is_none()
                    {
                        Some(UiAction::move_active_sprite(dir))
                    } else {
                        Some(UiAction::move_selected_square(dir, 1))
                    }
                } else {
                    Some(UiAction::change_selected_menu_item(dir))
                }
            }
            UserInput::Select => Some(UiAction::ChangeSelection),
            UserInput::AltDir(dir) => Some(UiAction::move_selected_square(dir, 2)),
            UserInput::Quit => Some(UiAction::quit()), // Might be able to just return None here
            UserInput::Debug => panic!("Debug state: {:?}", self),
            UserInput::Resize(bounds) => Some(UiAction::set_terminal_size(bounds)),
            UserInput::Activate => {
                if let Some(node) = self.game.node() {
                    if self.selection == Selection::SubMenu {
                        Some(UiAction::ConfirmSelection)
                    } else if self.selected_action_index().is_some() {
                        Some(UiAction::PerformSpriteAction)
                    } else {
                        let pt = self.selected_square();
                        let piece_opt = node.piece_at(pt);
                        if let Some(Piece::Program(_)) = piece_opt {
                            let piece_key = node.piece_key_at(pt).unwrap();
                            Some(UiAction::activate_sprite(piece_key))
                        } else {
                            None
                        }
                    }
                } else {
                    None
                }
            }
            UserInput::Click(pt) => self.action_for_char_pt(pt),
            _ => None,
        }
    }

    pub fn apply_action(&mut self, ui_action: UiAction) -> Result<(), String> {
        match ui_action {
            UiAction::ConfirmSelection => {
                self.selection = Selection::Grid;
                Ok(())
            }
            UiAction::ChangeSelectedMenuItem(dir) => {
                if let Some(node) = self.game.node() {
                    if self.selection == Selection::SubMenu {
                        let selected_sprite_key = node
                            .active_sprite_key()
                            .or_else(|| {
                                node.with_sprite_at(self.selected_square(), |sprite| (sprite.key()))
                            })
                            .unwrap();
                        if let Some(action_index) = self.selected_action_index() {
                            let num_actions = node
                                .with_sprite(selected_sprite_key, |sprite| sprite.actions().len())
                                .unwrap();
                            self.node_ui
                                .as_mut()
                                .unwrap()
                                .set_selected_action_index(match dir {
                                    Direction::North => {
                                        (action_index + num_actions - 1) % num_actions
                                    }
                                    Direction::South => (action_index + 1) & num_actions,
                                    _ => action_index,
                                })
                        }
                    }
                    Ok(())
                } else {
                    Err("No node".to_string())
                }
            }
            UiAction::ChangeSelection => {
                if let Some(node) = self.game.node() {
                    let selected_sprite_key = node.active_sprite_key().or_else(|| {
                        node.with_sprite_at(self.selected_square(), |sprite| (sprite.key()))
                    });
                    if self.selection == Selection::SubMenu {
                        if 0 != node
                            .with_sprite(selected_sprite_key.unwrap(), |sprite| sprite.moves())
                            .unwrap_or(0)
                        {
                            unsafe {
                                self.node_ui.as_mut().unwrap().clear_selected_action_index();
                            }
                        }
                        self.selection = Selection::Grid;
                    } else if self.selection == Selection::Grid {
                        if selected_sprite_key.is_some() {
                            if self.selected_action_index() == None {
                                self.set_default_selected_action();
                            }
                            self.selection = Selection::SubMenu;
                        }
                    }
                    Ok(())
                } else {
                    Err("No node".to_string())
                }
            }
            UiAction::MoveSelectedSquare { direction, speed } => {
                let range_limit = self.selected_action_index().and_then(|action_index| {
                    self.game.node().and_then(|node| {
                        node.with_active_sprite(|sprite| sprite.range_of_action(action_index))
                    })
                });
                self.move_selected_square(direction, speed, range_limit);
                Ok(())
            }
            UiAction::SetSelectedSquare(pt) => {
                self.set_selected_square(pt);
                Ok(())
            }
            UiAction::ActivateSprite(sprite_key) => {
                if self.game.active_sprite_key() == Some(sprite_key) {
                    self.game.deactivate_sprite();
                    Ok(())
                } else if self.game.activate_sprite(sprite_key) {
                    self.set_selected_square(
                        self.game.node().unwrap().grid().head(sprite_key).unwrap(),
                    );
                    Ok(())
                } else {
                    Ok(()) // Err("Trouble activating specified sprite".to_string())
                }
            }
            UiAction::DoGameAction(game_action) => self.game.apply_action(game_action),
            UiAction::SetTerminalSize(bounds) => {
                self.layout.resize(bounds);
                Ok(())
            }
            UiAction::Quit => {
                panic!("Thanks for playing")
            }
            UiAction::MoveActiveSprite(dir) => {
                if let Some(node) = self.game.node_mut() {
                    let (remaining_moves, head, is_tapped) = node
                        .with_active_sprite_mut(|mut sprite| {
                            (
                                sprite.move_sprite(vec![dir]),
                                sprite.head(),
                                sprite.tapped(),
                            )
                        })
                        .ok_or("No active sprite".to_string())?;

                    self.set_selected_square(head);

                    if remaining_moves? == 0 && !is_tapped && self.selected_action_index().is_none()
                    {
                        // Sprite is still active, must still have some moves
                        self.set_default_selected_action();
                    }
                } else {
                    unimplemented!("We don't have an implementation for world map yet")
                }
                Ok(())
            }
            UiAction::PerformSpriteAction => {
                if let Some(action_index) = self.selected_action_index() {
                    let result = self.game.node_mut().unwrap().perform_sprite_action(
                        action_index,
                        self.node_ui.as_ref().unwrap().selected_square(),
                    );
                    if result.is_some() {
                        unsafe {
                            self.node_ui.as_mut().unwrap().clear_selected_action_index();
                        }
                    }
                }
                Ok(())
            }
        }
    }
}

pub enum UiAction {
    ChangeSelection,
    ConfirmSelection,
    ChangeSelectedMenuItem(Direction),
    MoveSelectedSquare { direction: Direction, speed: usize },
    MoveActiveSprite(Direction),
    SetSelectedSquare(Point),
    DoGameAction(GameAction),
    PerformSpriteAction,
    ActivateSprite(usize),
    SetTerminalSize(Bounds),
    Quit,
}

impl UiAction {
    pub fn activate_sprite(sprite_key: usize) -> UiAction {
        UiAction::ActivateSprite(sprite_key)
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

    pub fn move_active_sprite(dir: Direction) -> UiAction {
        UiAction::MoveActiveSprite(dir)
    }

    pub fn change_selected_menu_item(dir: Direction) -> UiAction {
        UiAction::ChangeSelectedMenuItem(dir)
    }
}
