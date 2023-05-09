use old_game_core::{
    error::Error, event::Event, EventLog, EventPublisher, GameCommand, GameState, Informant,
};
use std::{collections::VecDeque, time::Duration};

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
    action_vec: VecDeque<UiAction>,
}

impl CrosstermInformant {
    const MAX_EVENTS_IN: usize = 10;

    pub fn new(state: &GameState) -> Self {
        let (t_width, t_height) =
            crossterm::terminal::size().expect("Problem getting terminal size");

        let informant = CrosstermInformant {
            node_ui: state.node().map(NodeUiState::from),
            layout: Layout::new((t_width, t_height).into()),
            draw_config: DrawConfiguration::default(),
            super_state: SuperState::from(state),
            action_vec: Default::default(),
        };
        informant.render(state);
        informant
    }
}

impl CrosstermInformant {
    fn render(&self, game_state: &GameState) {
        self.super_state.render(game_state).unwrap();
    }

    fn queue_up_ui_actions(&mut self, game_state: &GameState) {
        for _ in 0..Self::MAX_EVENTS_IN {
            if crossterm::event::poll(Duration::from_secs(0)).unwrap() {
                let e = crossterm::event::read().unwrap();
                if let Some(input) = UserInput::from_event(e) {
                    for action in self.super_state.ui_actions_for_input(game_state, input) {
                        self.action_vec.push_back(action)
                    }
                }
            } else {
                break;
            }
        }
    }
}

impl Informant for CrosstermInformant {
    fn tick(&mut self, game_state: &GameState) -> Vec<GameCommand> {
        self.queue_up_ui_actions(game_state);
        if self.action_vec.is_empty() {
            return vec![];
        }
        match self.action_vec.front() {
            Some(UiAction::GameCommand(gc)) => {
                return vec![gc.clone()];
            }
            Some(_) => {
                let ui_action = self.action_vec.pop_front().unwrap();
                self.super_state
                    .apply_action(ui_action, game_state)
                    .unwrap();
                self.render(game_state);
            }
            None => {}
        }
        return vec![];
    }

    fn collect(&mut self, event: &Event, _game_state: &GameState) {}

    fn fail(&mut self, error: &Error, command: &GameCommand, _game_state: &GameState) {
        let ui_action = UiAction::GameCommand(command.clone());
        if self.action_vec.front() == Some(&ui_action) {
            self.action_vec.pop_front();
        }
    }

    fn publish(&mut self, command: &GameCommand, game_state: &GameState) {
        let ui_action = UiAction::GameCommand(command.clone());
        if self.action_vec.front() == Some(&ui_action) {
            self.action_vec.pop_front();
        }
        self.super_state
            .apply_action(ui_action, game_state)
            .unwrap();

        self.render(game_state);
    }

    fn collect_undo(&mut self, event: &Event, _game_state: &GameState, _event_log: &EventLog) {}
}
