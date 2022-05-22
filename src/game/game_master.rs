use super::{event::{Event, Change, EventErr}, EnemyAi, GameAction, GameState, NodeChange, GameChange};
use std::sync::mpsc::{channel, Receiver};

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
    ai_action_receiver: Option<Receiver<GameAction>>, // events: List of events
                                                      // caching of advance-states
    last_event_id: usize
}

impl AuthorityGameMaster {
    pub fn apply<C: Into<Change>>(&mut self, change: C) -> Result<(), CommandError> {
        let new_event_id = self.last_event_id + 1;
        let wrapped_change = change.into();
        let result = wrapped_change.apply(new_event_id, &mut self.state); // Add event number and record
        log::debug!("Event result: {:?}", result);
        if let Err(err) = result {
            Err(CommandError::NodeActionError(format!("{:?} error occurred while performing change {:?}", err, wrapped_change)))
        } else {
            self.last_event_id = new_event_id;
            Ok(())
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

    // Util to ease transition to command
    // Should be easy to replace "GameAction" with "GameCommand"
    fn apply_game_action(&mut self, action: GameAction) -> Result<(), CommandError> {
        match action {
            GameAction::ActivateSprite(sprite_key) => {
                self.apply(NodeChange::ActivateSprite(sprite_key))?;
            }
            GameAction::DeactivateSprite => {
                self.apply(NodeChange::DeactivateSprite)?;
                let node = self.state.node()
                    .expect("How could this not exist if DeactivateSprite successful?");

                if node.untapped_sprites_on_active_team() == 0 {
                    self.apply(NodeChange::FinishTurn)?;
                    self.check_to_run_ai();
                }
            }
            GameAction::TakeSpriteAction(action_index, pt) => {
                self.apply(NodeChange::TakeSpriteAction(action_index, pt))?;
                let node = self.state.node()
                    .expect("How could this not exist if TakeSpriteAction successful?");

                if node.untapped_sprites_on_active_team() == 0 {
                    self.apply(NodeChange::FinishTurn)?;
                    self.check_to_run_ai();
                }
            }
            GameAction::MoveActiveSprite(directions) => {
                self.apply(NodeChange::MoveActiveSprite(directions[0]))?;
            }
            GameAction::Next => {
                unimplemented!("Should not implement: Is already handled as GameCommand")
            }
        }
        Ok(())

    }

    // TODO should return event?
    pub fn apply_command(&mut self, command: GameCommand) -> Result<(), CommandError> {
        match command {
            GameCommand::PlayerNodeAction(action) => self.apply_game_action(action),
            GameCommand::Next => {
                if let Some(rx) = &self.ai_action_receiver {
                    let action = rx.recv().unwrap();

                    self.apply_game_action(action)
                } else {
                    self.apply(GameChange::NextPage)
                }
            }
            GameCommand::Skip => {
                unimplemented!("Skip action not yet implemented");
            }
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
            last_event_id: 0,
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
