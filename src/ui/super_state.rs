use super::{DrawConfiguration, Layout, UserInput};
use crate::{Bounds, Direction, GameAction, GameState, Node, Piece, Point, PointSet};

// TODO NodeUiState
#[derive(Debug)]
pub struct SuperState {
    pub game: GameState,
    layout: Layout,
    draw_config: DrawConfiguration,
    terminal_size: (usize, usize),
    selected_square: Point, // Might be a property of layout?
    selected_action_index: Option<usize>,
    selection: Selection,
}

#[derive(Debug)]
enum Selection {
    Grid = 0,
    PauseMenu = 1,
    SubMenu = 2,
    SubMenu2 = 3,
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

    fn set_default_selected_action(&mut self) {
        // TODO check sprite metadata for last selected action?
        self.selected_action_index = Some(0);
    }
    pub fn move_selected_square(
        &mut self,
        direction: Direction,
        speed: usize,
        range_limit: Option<PointSet>,
    ) {
        let new_pt = direction.add_to_point(
            self.selected_square,
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
        self.selected_square = new_pt;
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
            UserInput::Dir(dir) => {
                if self.game.active_sprite_key().is_some() && self.selected_action_index().is_none()
                {
                    Some(UiAction::move_active_sprite(dir))
                } else {
                    Some(UiAction::move_selected_square(dir, 1))
                }
            }
            UserInput::AltDir(dir) => Some(UiAction::move_selected_square(dir, 2)),
            UserInput::Quit => Some(UiAction::quit()), // Might be able to just return None here
            UserInput::Debug => panic!("Debug state: {:?}", self),
            UserInput::Resize(bounds) => Some(UiAction::set_terminal_size(bounds)),
            UserInput::Activate => {
                if let Some(node) = self.game.node() {
                    if self.selected_action_index.is_some() {
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
            UiAction::MoveSelectedSquare { direction, speed } => {
                let range_limit = self.selected_action_index().and_then(|action_index| {
                    self.game.node().and_then(|node| {
                        node.with_active_sprite_wrapped(|sprite| {
                            sprite.range_of_action(action_index)
                        })
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
                        .with_active_sprite_mut_wrapped(|mut sprite| {
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
                    let result = self
                        .game
                        .node_mut()
                        .unwrap()
                        .perform_sprite_action(action_index, self.selected_square);
                    if result.is_some() {
                        self.selected_action_index = None;
                    }
                }
                Ok(())
            }
        }
    }
}

pub enum UiAction {
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
}
