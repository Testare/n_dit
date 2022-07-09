use game_core::{
    error::Error, event::Event, EventLog, EventPublisher, GameCommand, GameState, Informant,
};
use std::time::Duration;

use super::{DrawConfiguration, Layout, NodeUiState, SuperState, UiAction, UserInput};

#[derive(Debug)]
pub struct TuiEventPublisher();

impl EventPublisher for TuiEventPublisher {
    fn collect(&mut self, event: &Event, _game_state: &GameState) {
        log::debug!(" TEP collect {:?}", event)
    }
    fn fail(&mut self, error: &Error, _command: &GameCommand) {
        log::debug!(" TEP fail {:?}", error)
    }
    fn publish(&mut self, command: &GameCommand) {
        log::debug!(" TEP publish {:?}", command)
    }
    fn collect_undo(&mut self, event: &Event, _game_state: &GameState, _event_log: &EventLog) {
        log::debug!(" TEP undo {:?}", event)
    }
}

#[derive(Debug)]
pub struct CrosstermInformant {
    layout: Layout,
    draw_config: DrawConfiguration,
    node_ui: Option<NodeUiState>,
    super_state: SuperState,
}

impl CrosstermInformant {
    pub fn new(state: &GameState) -> Self {
        let (t_width, t_height) =
            crossterm::terminal::size().expect("Problem getting terminal size");
        log::debug!("LOGWOO");

        let informant = CrosstermInformant {
            node_ui: state.node().map(NodeUiState::from),
            layout: Layout::new((t_width, t_height).into()),
            draw_config: DrawConfiguration::default(),
            super_state: SuperState::from(state),
        };
        informant.render(state);
        informant
    }
}

impl CrosstermInformant {
    fn render(&self, game_state: &GameState) {
        self.super_state.render(game_state);
    }
}

impl Informant for CrosstermInformant {
    fn tick(&mut self, game_state: &GameState) -> Option<GameCommand> {
        if crossterm::event::poll(Duration::from_secs(0)).unwrap() {
            let e = crossterm::event::read().unwrap();
            if let Some(input) = UserInput::from_event(e) {
                let mut game_command = None;
                for action in self.super_state.ui_actions_for_input(game_state, input) {
                    if let UiAction::GameCommand(gc) = action {
                        game_command = Some(gc);
                    } else {
                        self.super_state.apply_action(action, game_state).unwrap();
                        self.render(game_state);
                    }
                }
                game_command
            } else {
                None
            }
        } else {
            None
        }
    }
    fn collect(&mut self, event: &Event, _game_state: &GameState) {
        log::debug!(" TEP collect {:?}", event)
    }
    fn fail(&mut self, error: &Error, _command: &GameCommand, _game_state: &GameState) {
        log::debug!(" TEP fail {:?}", error)
    }
    fn publish(&mut self, command: &GameCommand, game_state: &GameState) {
        self.super_state
            .apply_action(UiAction::GameCommand(command.clone()), game_state)
            .unwrap();
        log::debug!(" TEP publish {:?}", command);
        self.render(game_state);
    }

    fn collect_undo(&mut self, event: &Event, _game_state: &GameState, _event_log: &EventLog) {
        log::debug!(" TEP undo {:?}", event)
    }
}
