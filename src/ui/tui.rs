use game_core::{error::Error, event::Event, Informant, EventLog, EventPublisher, GameCommand, GameState};
use std::time::Duration;

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


#[derive(Debug, Default)]
pub struct CrosstermInformant {

}

impl CrosstermInformant {

    pub fn new() -> Self {
        Default::default()
    }
}

impl Informant for CrosstermInformant {
    fn poll(&self, game_state: &GameState) -> Option<GameCommand> {
        if crossterm::event::poll(Duration::from_secs(0)).unwrap() {
            let e = crossterm::event::read().unwrap();
            if let crossterm::event::Event::Key(crossterm::event::KeyEvent { code , ..}) = e {
                if code == crossterm::event::KeyCode::Char('q') {
                    return Some(GameCommand::ShutDown);
                }
            }
            Some(GameCommand::Start)
        } else {
            None
        }
    }
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