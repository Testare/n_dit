use super::{Direction, Node, Point, WorldMap};

pub struct GameState {
    _world_map: WorldMap,
    node: Option<Node>,
}

impl GameState {
    pub fn node(&self) -> Option<&Node> {
        self.node.as_ref()
    }

    pub fn from(node: Option<Node>) -> Self {
        GameState {
            node,
            _world_map: WorldMap { nodes: 1 },
        }
    }

    pub fn waiting_on_player_input(&self) -> bool {
        unimplemented!("TODO: Return true when waiting for player action. False indicates that the game state should update itself without player input before the next loop")
    }

    pub fn apply_action(&mut self, game_action: GameAction) -> Result<(), String> {
        if self.waiting_on_player_input() {
            match game_action {
                GameAction::Next => Err(String::from("Waiting for player input")),
                GameAction::ActivateSprite(sprite_key) => self
                    .node
                    .as_mut()
                    .ok_or(String::from(
                        "Action doesn't make sense when we're not in a node",
                    ))
                    .and_then(|node| {
                        if node.activate_sprite(sprite_key) {
                            Ok(())
                        } else {
                            Err(String::from("Sprite does not exist"))
                        }
                    }),
                _ => unimplemented!("TODO other game actions"),
            }
        } else {
            if let GameAction::Next = game_action {
                unimplemented!("TODO update state")
            } else {
                Err(String::from(
                    "Cannot accept player actions right now, next action must be 'Next'",
                ))
            }
        }
    }
}

#[derive(Clone, Eq, PartialEq)]
pub enum GameAction {
    Next,                  // when we're not waiting on player_input, go to next action.
    ActivateSprite(usize), // Starts using a unit.
    MoveActiveSprite(Vec<Direction>),
    TakeSpriteAction(usize, Point),
}
