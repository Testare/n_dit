use game_core::{error::Error, event::Event, EventLog, EventPublisher, GameCommand, GameState};

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
