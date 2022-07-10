use std::fmt::Debug;
use crate::{
    GameState,
    error::Error,

};
use super::{
    GameCommand,
    Event,
    EventLog,
};
/**
 * Informant metaphor:
 * 
 * Like a spy, they go in and are the eyes on the inside, and usually
 * say what they are told to.
 * 
 * TODO Nevermind let's just call this a player? Or at least, map one informant
 * to one player. I don't think we can
 * support multiple players on a single screen without doing split screen,
 * it'll involve messing .too much with game logic.
 * 
 * For split screen, we could just have two informants, one for each screen.
 * 
 * Perhaps a better name would be "UI Pipe", since it it's representative of
 */
pub trait Informant: Debug {
    // Should be updated to Results
    fn tick(&mut self, game_state: &GameState) -> Option<GameCommand>;
    fn collect(&mut self, event: &Event, game_state: &GameState);
    fn fail(&mut self, error: &Error, command: &GameCommand, game_state: &GameState);
    fn publish(&mut self, command: &GameCommand, game_state: &GameState);
    fn collect_undo(&mut self, event: &Event, game_state: &GameState, event_log: &EventLog);
}
