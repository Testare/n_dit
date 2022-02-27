use super::{DrawConfiguration, Layout, NodeUiState, UserInput};
use crate::{Bounds, Direction, GameAction, GameState, Node, Piece, Point, PointSet, Team};

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
        let selected_sprite_key = self
            .game
            .node()
            .unwrap()
            .with_sprite_at(new_pt, |sprite| sprite.key());
        if let Some(node_ui) = &mut self.node_ui {
            node_ui.set_selected_sprite_key_if_phase_is_right(selected_sprite_key)
        }

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
        self.node_ui
            .as_mut()
            .zip(self.game.node_mut())
            .map(|(node_ui, node)| node_ui.apply_action(node, ui_action.clone()))
            .unwrap_or(Err("Node UI action, but no node".to_string()))?;

        match &ui_action {
            UiAction::MoveSelectedSquare { direction, speed } => {
                let range_limit = self.selected_action_index().and_then(|action_index| {
                    self.game.node().and_then(|node| {
                        node.with_active_sprite(|sprite| sprite.range_of_action(action_index))
                    })
                });
                self.move_selected_square(*direction, *speed, range_limit);
            }
            UiAction::SetTerminalSize(bounds) => {
                self.layout.resize(*bounds);
            }
            UiAction::Quit => {
                panic!("Thanks for playing")
            }
            // Should be moved into the GameAction
            UiAction::PerformSpriteAction | UiAction::ActivateSprite(_) => {
                if let Some(node) = self.game.node_mut() {

                    let enemy_sprites_remaining = node
                        .filtered_sprite_keys(|_, sprite| sprite.team() == Team::EnemyTeam)
                        .len();
                    if enemy_sprites_remaining == 0 {
                        panic!("No enemies remain! You win!")
                    }
                    let untapped_player_sprites_remaining = node
                        .filtered_sprite_keys(|_, sprite| sprite.team() == Team::PlayerTeam && !sprite.tapped())
                        .len();
                    
                    if untapped_player_sprites_remaining == 0 {
                        node.change_active_team();
                        if node.active_team() == Team::EnemyTeam {
                            let enemy_ai_actions = node.enemy_ai().generate_animation(node);
                            self.game.set_animation(enemy_ai_actions);
                        }
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }
}

#[derive(Clone)]
pub enum UiAction {
    ChangeSelection,
    ConfirmSelection,
    ChangeSelectedMenuItem(Direction),
    // Should be a GameAction
    MoveSelectedSquare { direction: Direction, speed: usize },
    // Should be a GameAction
    MoveActiveSprite(Direction),
    SetSelectedSquare(Point),
    GameAction(GameAction),
    // Should be a GameAction
    PerformSpriteAction,
    // Should be a GameAction
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
        UiAction::GameAction(GameAction::next())
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
