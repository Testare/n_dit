use super::{EnemyAi, GameAction, GameState};
use std::sync::mpsc::{channel, Receiver};

// An intermediary between the Users and the persistent game state. As such it fulfills the following roles:
// * Behavior of AI players, including the rendering of incomplete-but-finalized
// AI behavior so we don't have to wait for the AI to complete thinking before
// start rendering.
// * Caching and advance-calculating of animated states
// * Event listening/Registering UI Handlers
// * Behavior with controllers over a network.
// * Translation of player/system input "commands" to "events"
//
// Might want to add a feature flag for network behavior?
// TODO Create generic "GameMaster" trait for network game masters?

#[derive(Debug)]
pub struct AuthorityGameMaster {
    state: GameState,
    ai_action_receiver: Option<Receiver<GameAction>>, // events: List of events
                                                      // caching of advance-states
}

impl AuthorityGameMaster {
    /// Utility method while refactoring to ease transition to Commands
    /// TODO remove
    #[deprecated]
    pub fn apply_node_action(&mut self, action: GameAction) -> Result<(), String> {
        self.apply_command(GameCommand::PlayerNodeAction(action))
            .map_err(|_| "Node action failure".to_string())
    }

    fn check_to_run_ai(&mut self) {
        if let Some(node) = self.state.node() {
            if self.ai_action_receiver.is_none() && node.active_team().is_ai() {
                let (tx, rx) = channel();
                self.ai_action_receiver = Some(rx);
                let node_destructble = node.clone();
                std::thread::spawn(move || {
                    let ai: EnemyAi = *node_destructble.enemy_ai();
                    ai.generate_enemy_ai_actions(node_destructble, |action| {
                        tx.send(action).unwrap()
                    })
                });
            } else if self.ai_action_receiver.is_some() && !node.active_team().is_ai() {
                // This might cause some bugs if this method isn't run between enemy turns
                self.ai_action_receiver = None;
            }
        }
    }

    // TODO should return event?
    pub fn apply_command(&mut self, command: GameCommand) -> Result<(), CommandError> {
        match command {
            GameCommand::Next => {
                if let Some(rx) = &self.ai_action_receiver {
                    let action = rx.recv().unwrap();

                    self.state
                        .apply_action(&action)
                        .map_err(CommandError::NodeActionError)?;
                    self.check_to_run_ai();
                    Ok(())
                } else {
                    self.state
                        .apply_action(&GameAction::next())
                        .map_err(CommandError::NodeActionError)
                }
            }
            GameCommand::Skip => {
                unimplemented!("Skip action not yet implemented");
            }
            GameCommand::PlayerNodeAction(action) => {
                self.state
                    .apply_action(&action)
                    .map_err(CommandError::NodeActionError)?;
                self.check_to_run_ai();
                Ok(())
            },
            GameCommand::Undo => {
                unimplemented!("Skip action not yet implemented");
            }
        }
        // Record events
        // Trigger event listeners
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
        }
    }
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub enum GameCommand {
    Next,
    Skip,
    PlayerNodeAction(GameAction),
    Undo,
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub enum CommandError {
    #[deprecated]
    NodeActionError(String),
    /*InvalidCommand(String),
    ImpossibleCommand(String),
    FailedCommand(String)*/
}

impl ToString for CommandError {
    fn to_string(&self) -> String {
        match self {
            CommandError::NodeActionError(str) => str.to_owned(),
        }
    }
}
