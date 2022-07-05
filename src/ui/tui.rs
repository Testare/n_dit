use game_core::{EventPublisher, event::Event, error::Error, EventLog, GameState, GameCommand};

#[derive(Debug)]
pub struct TuiEventPublisher();

impl EventPublisher for TuiEventPublisher {
    fn collect(&mut self, event: &Event, game_state: &GameState) {
        log::debug!(" TEP collect {:?}", event)
    }
    fn fail(&mut self, error: &Error, command: &GameCommand) {
        log::debug!(" TEP fail {:?}", error)

    }
    fn publish(&mut self, command: &GameCommand) {
        log::debug!(" TEP publish {:?}", command)
    }
    fn collect_undo(&mut self, event: &Event, game_state: &GameState, event_log: &EventLog) {
        log::debug!(" TEP undo {:?}", event)
    }
}
