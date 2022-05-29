mod game_command;

use super::error::{Error, Result};
use super::{
    event::{Change, Event},
    EnemyAi, GameState,
};
use std::collections::HashMap;
use std::sync::mpsc::{channel, Receiver};

pub use game_command::GameCommand;

// An intermediary between the Users and the persistent game state. As such it fulfills the following roles:
// * Behavior of AI players, including the rendering of incomplete-but-finalized
// AI behavior so we don't have to wait for the AI to complete thinking before
// start rendering.
// * Caching and advance-calculating of animated states
// * Behavior with controllers over a network.
// * Translation of player/system input "commands" to "events"
//
// Might want to add a feature flag for network behavior?
// TODO Create generic "GameMaster" trait for network game masters?

#[derive(Debug)]
pub struct AuthorityGameMaster {
    state: GameState,
    ai_action_receiver: Option<Receiver<Change>>, // caching of advance-states
    event_log: EventLog,
    event_publishers: EventPublisherManager,
}

impl AuthorityGameMaster {
    // Used in GameCommand
    fn apply<C: Into<Change>>(&mut self, change: C) -> Result<()> {
        let new_event_id = self.event_log.last_event_id() + 1;
        let wrapped_change = change.into();
        let result = wrapped_change.apply(new_event_id, &mut self.state); // Add event number and record
        log::debug!("Event result: {:?}", result);
        match result {
            Err(err) => Err(err),
            Ok(event) => {
                self.event_publishers.collect(&event, &self.state);
                self.event_log.push_event(event);
                Ok(())
            }
        }
    }

    fn check_to_run_ai(&mut self) {
        if let Some(node) = self.state.node() {
            if self.ai_action_receiver.is_none() && node.active_team().is_ai() {
                let (tx, rx) = channel();
                self.ai_action_receiver = Some(rx);
                let node_destructble = node.clone();
                std::thread::spawn(move || {
                    let ai: EnemyAi = *node_destructble.enemy_ai();
                    ai.generate_enemy_ai_actions(node_destructble, |change| {
                        tx.send(change.into()).unwrap()
                    })
                });
            } else if self.ai_action_receiver.is_some() && !node.active_team().is_ai() {
                // This might cause some bugs if this method isn't run between enemy turns
                self.ai_action_receiver = None;
            }
        }
    }

    pub fn add_publisher<P: EventPublisher + 'static>(&mut self, key: String, publisher: P) {
        self.event_publishers.add_publisher(key, publisher);
    }

    pub fn remove_publisher(&mut self, key: String) {
        self.event_publishers.remove_publisher(key);
    }

    pub fn apply_command(&mut self, command: GameCommand) -> Result<()> {
        let result = game_command::apply_command_dispatch(self, &command);
        match &result {
            Ok(_) => self.event_publishers.publish(&command),
            // Failing here instead of in apply in case the command wants to modify the error message a little.
            Err(error) => self.event_publishers.fail(error, &command),
        }
        result
    }

    // TEMPORARY
    pub fn state(&self) -> &GameState {
        &self.state
    }
}

impl From<GameState> for AuthorityGameMaster {
    fn from(state: GameState) -> AuthorityGameMaster {
        AuthorityGameMaster {
            state,
            ai_action_receiver: None,
            event_log: EventLog::default(),
            event_publishers: EventPublisherManager::default(),
        }
    }
}

pub trait EventPublisher: std::fmt::Debug {
    fn collect(&mut self, event: &Event, game_state: &GameState);
    fn fail(&mut self, error: &Error, command: &GameCommand);
    fn publish(&mut self, command: &GameCommand);
    // fn collect_undo(&mut self, event: &Event, game_state: &GameState, event_log: &EventLog);
}

#[derive(Debug, Default)]
struct EventPublisherManager {
    publishers: HashMap<String, Box<dyn EventPublisher>>,
}

impl EventPublisherManager {
    fn add_publisher<P: EventPublisher + 'static>(
        &mut self,
        key: String,
        publisher: P,
    ) -> Option<Box<dyn EventPublisher>> {
        self.publishers.insert(key, Box::new(publisher))
    }

    fn remove_publisher(&mut self, key: String) -> Option<Box<dyn EventPublisher>> {
        self.publishers.remove(&key)
    }

    fn collect(&mut self, event: &Event, game_state: &GameState) {
        for publisher in self.publishers.values_mut() {
            publisher.collect(event, game_state);
        }
    }

    fn fail(&mut self, error: &Error, command: &GameCommand) {
        for publisher in self.publishers.values_mut() {
            publisher.fail(error, command);
        }
    }

    fn publish(&mut self, command: &GameCommand) {
        for publisher in self.publishers.values_mut() {
            publisher.publish(command);
        }
    }
}

#[derive(Debug, Default, Clone)]
struct EventLog(Vec<Event>);

impl EventLog {
    fn push_event(&mut self, event: Event) {
        self.0.push(event);
    }

    fn last_event_id(&self) -> usize {
        self.0.last().map(Event::id).unwrap_or(0)
    }
}
