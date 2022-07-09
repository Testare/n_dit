mod game_command;
mod informant;

pub use informant::Informant;

use super::error::{Error, ErrorMsg as _, Result};
use super::{
    event::{Change, Event},
    EnemyAi, GameState,
};
use std::collections::HashMap;
use std::sync::mpsc::{channel, Receiver};
use std::time::{Duration, Instant};

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
    running: bool,
    state: GameState,
    ai_action_receiver: Option<Receiver<Change>>, // caching of advance-states
    event_log: EventLog,
    event_publishers: EventPublisherManager,
    informants: InformantManager,
}

// Currently render loop is coupled with input loop and animation loop.
// In the future all 3 of these need to be decoupled
const FRAME_DELAY: Duration = Duration::from_millis(100);

impl AuthorityGameMaster {

    pub fn informants_testing(&mut self) -> &mut InformantManager {
        &mut self.informants
    }

    pub fn setup_informant<I: Informant + 'static, C: FnOnce(&GameState)-> I>(&mut self, construct_informant: C) {
        self.informants.add_informant(construct_informant(&self.state));
    }

    pub fn run(&mut self) {
        let mut start_frame;
        let mut frame_count: usize = 0;
        while self.running {
            start_frame = Instant::now();
            // log::info!("Frame Time: {:?}", start_frame);
            let commands = self.informants.tick(&self.state);
            for (_player_id, gc) in commands {
                if gc == GameCommand::ShutDown {
                    self.running = false;
                } else if let Err(error) = self.apply_command(gc) {
                    self.running = !error.is_critical()
                }
            }
            if frame_count % 5 == 0 {
                self.apply_command(GameCommand::Next); // TODO better AI command logic
            }
            frame_count = (frame_count + 1) % 2100;
            let time_passed = Instant::now() - start_frame;
            std::thread::sleep(FRAME_DELAY - time_passed);
        }
    }
    // Used in GameCommand
    fn apply<C: Into<Change>>(&mut self, change: C) -> Result<()> {
        let new_event_id = self.event_log.last_event_id() + 1;
        let wrapped_change = change.into();
        let result = wrapped_change.apply(new_event_id, &mut self.state); // Add event number and record
        log::debug!("Event result: {:?}", result);
        match result {
            Err(err) => Err(err),
            Ok(event) => {
                self.informants.collect(&event, &self.state);
                self.event_log.push_event(event);
                Ok(())
            }
        }
    }

    fn undo(&mut self) -> Result<()> {
        let event_opt = self.event_log.pop_event();
        if let Some(event) = event_opt {
            log::debug!("Undoing event {:?}", &event);
            event.undo(&mut self.state)?;
            self.informants
                .collect_undo(&event, &self.state, &self.event_log);
            Ok(())
        } else {
            "Nothing to undo".invalid()
        }
    }

    fn undo_until_last_durable_event(&mut self) -> Result<()> {
        self.undo()?;
        while !self.event_log.is_durable() {
            self.undo()?;
        }
        Ok(())
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

    pub fn add_publisher<P: EventPublisher + 'static>(&mut self, key: &str, publisher: P) {
        self.event_publishers.add_publisher(key, publisher);
    }

    pub fn remove_publisher(&mut self, key: String) {
        self.event_publishers.remove_publisher(key);
    }

    pub fn apply_command(&mut self, command: GameCommand) -> Result<()> {
        let result = game_command::apply_command_dispatch(self, &command);
        let AuthorityGameMaster{informants, state, ..} = self;
        match &result {
            Ok(_) => informants.publish(&command, state),
            // Failing here instead of in apply in case the command wants to modify the error message a little.
            Err(error) => informants.fail(error, &command, state),
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
            running: true,
            ai_action_receiver: None,
            event_log: EventLog::default(),
            event_publishers: EventPublisherManager::default(),
            informants: Default::default(),
        }
    }
}

pub trait EventPublisher: std::fmt::Debug {
    fn collect(&mut self, event: &Event, game_state: &GameState);
    fn fail(&mut self, error: &Error, command: &GameCommand);
    fn publish(&mut self, command: &GameCommand);
    fn collect_undo(&mut self, event: &Event, game_state: &GameState, event_log: &EventLog);
}

#[derive(Debug, Default)]
struct EventPublisherManager {
    publishers: HashMap<String, Box<dyn EventPublisher>>,
}

impl EventPublisherManager {
    fn add_publisher<P: EventPublisher + 'static>(
        &mut self,
        key: &str,
        publisher: P,
    ) -> Option<Box<dyn EventPublisher>> {
        self.publishers.insert(key.to_string(), Box::new(publisher))
    }

    fn remove_publisher(&mut self, key: String) -> Option<Box<dyn EventPublisher>> {
        self.publishers.remove(&key)
    }

    fn collect(&mut self, event: &Event, game_state: &GameState) {
        for publisher in self.publishers.values_mut() {
            publisher.collect(event, game_state);
        }
    }

    fn collect_undo(&mut self, event: &Event, game_state: &GameState, event_log: &EventLog) {
        for publisher in self.publishers.values_mut() {
            publisher.collect_undo(event, game_state, event_log);
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
pub struct EventLog(Vec<Event>);

impl EventLog {
    fn pop_event(&mut self) -> Option<Event> {
        self.0.pop()
    }

    fn push_event(&mut self, event: Event) {
        self.0.push(event);
    }

    fn last_event_id(&self) -> usize {
        self.0.last().map(Event::id).unwrap_or(0)
    }

    fn is_durable(&self) -> bool {
        self.0.last().map(Event::is_durable).unwrap_or(true)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct InformantId(usize);

// TODO This should perhaps not be public
#[derive(Debug, Default)]
pub struct InformantManager {
    informants: HashMap<InformantId, Box<dyn Informant>>,
    informant_id_counter: usize
}

impl InformantManager {

    fn tick(&mut self, state: &GameState) -> Vec<(InformantId, GameCommand)> {
        self.informants.iter_mut().filter_map(|(informant_id, informant)| {
            Some((*informant_id, informant.tick(state)?))
        }).collect()
    }

    pub fn add_informant<P: Informant + 'static>(
        &mut self,
        informant: P,
    ) -> Option<Box<dyn Informant>> {
        self.informant_id_counter += 1;
        self.informants.insert(InformantId(self.informant_id_counter), Box::new(informant))
    }

    fn remove_informant(&mut self, id: &InformantId) -> Option<Box<dyn Informant>> {
        self.informants.remove(id)
    }

    fn collect(&mut self, event: &Event, game_state: &GameState) {
        for informant in self.informants.values_mut() {
            informant.collect(event, game_state);
        }
    }

    fn collect_undo(&mut self, event: &Event, game_state: &GameState, event_log: &EventLog) {
        for informant in self.informants.values_mut() {
            informant.collect_undo(event, game_state, event_log);
        }
    }

    fn fail(&mut self, error: &Error, command: &GameCommand, state: &GameState) {
        for informant in self.informants.values_mut() {
            informant.fail(error, command, state);
        }
    }

    fn publish(&mut self, command: &GameCommand, state: &GameState) {
        for informant in self.informants.values_mut() {
            informant.publish(command, state);
        }
    }
}
