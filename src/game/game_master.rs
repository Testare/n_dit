use super::{GameState, GameAction, EnemyAiAction, EnemyAi};
use std::sync::mpsc::{channel,Receiver, Sender};

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
    ai_action_receiver: Option<Receiver<EnemyAiAction>>
    // events: List of events
    // caching of advance-states
}

impl AuthorityGameMaster {
    /// Utility method while refactoring to ease transition to Commands
    /// TODO remove
    #[deprecated]
    pub fn apply_node_action(&mut self, action: GameAction) -> Result<(), String> {
        self.apply_command(GameCommand::PlayerNodeAction(action))
            .map_err(|_|"Node action failure".to_string())
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
                self.ai_action_receiver = None;
            }
        }
    }

    // TODO should return event?
    pub fn apply_command(&mut self, command: GameCommand) -> Result<(), CommandError> {
        match command {
            GameCommand::Next => {
                if let Some(rx) = &self.ai_action_receiver {
                    let aiaction = rx.recv().unwrap();
                    let action = match aiaction {
                        EnemyAiAction::ActivateSprite(sprite_key) => GameAction::activate_sprite(sprite_key),
                        EnemyAiAction::MoveSprite(dir) => GameAction::move_active_sprite(vec![dir]),
                        EnemyAiAction::PerformNoAction => GameAction::deactivate_sprite(),
                        EnemyAiAction::PerformAction(action_index, pt) => GameAction::take_sprite_action(action_index, pt)
                    };
                    
                    self.state.apply_action(&action)
                        .map_err(|s|CommandError::NodeActionError(s))?;
                    self.check_to_run_ai();
                    Ok(())
                } else {
                    self.state.apply_action(&GameAction::next())
                        .map_err(|s|CommandError::NodeActionError(s))
                }
            },
            GameCommand::Skip => {
                unimplemented!("Skip action not yet implemented");
            },
            GameCommand::PlayerNodeAction(action) => {
                self.state.apply_action(&action)
                    .map_err(|s|CommandError::NodeActionError(s))?;
                self.check_to_run_ai();
                Ok(())
            },
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
            ai_action_receiver: None
        }
    }
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub enum GameCommand {
    Next,
    Skip,
    PlayerNodeAction(GameAction)
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub enum CommandError {
    NodeActionError(String)
}

impl ToString for CommandError {
    fn to_string(&self) -> String {
        match self {
            CommandError::NodeActionError(str) => str.to_owned()
        }
    }
}


trait EventListener {

}