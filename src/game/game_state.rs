use super::{Direction, Node, Point, WorldMap};

#[derive(Debug)]
pub struct GameState {
    _world_map: WorldMap,
    node: Option<Node>,
}

impl GameState {
    pub fn node(&self) -> Option<&Node> {
        self.node.as_ref()
    }

    pub fn node_mut(&mut self) -> Option<&mut Node> {
        self.node.as_mut()
    }

    pub fn perform_sprite_action(&mut self, sprite_action_index: usize, target_pt: Point) -> Option<()> {
        let node = self.node.as_mut()?;
        let active_sprite_key = node.active_sprite_key()?;
        let action = node.with_sprite(active_sprite_key, |sprite| sprite.actions().get(sprite_action_index).map(|action|action.unwrap())).flatten()?;
        let result = action.apply(self, active_sprite_key, target_pt);
        match result {
            Ok(()) => {
                self.node.as_mut().unwrap().deactivate_sprite();
                Some(())
            },
            _ => None
        }
    }

    /// Returns remaining moves
    /// Perhaps should be a function on NODE
    pub fn move_active_sprite(&mut self, directions: Vec<Direction>) -> Result<usize, String> {
        // TODO instead of invoking grid_map functions directly, use Node as an interface
        // TODO refactor main logic to "move_sprite(sprite_key, direction) -> Result<usize>" where usize is remaining moves
        let node = self.node.as_mut().ok_or("No node".to_string())?;
        let sprite_key = node
            .active_sprite_key()
            .ok_or("No active sprite".to_string())?;
        if node
            .with_sprite(sprite_key, |sprite| sprite.moves() == 0 || sprite.tapped())
            .unwrap()
        {
            return Err("Sprite cannot move".to_string());
        }
        let bounds = node.bounds();
        let mut size = node.grid().len_of(sprite_key);
        let (mut remaining_moves, max_size) = node
            .with_sprite(sprite_key, |sprite| (sprite.moves(), sprite.max_size()))
            .unwrap();

        for dir in directions {
            let head = node.grid().head(sprite_key).unwrap();
            let next_pt = dir.add_to_point(head, 1, bounds);
            let sucessful_movement = node.grid_mut().push_front(next_pt, sprite_key);
            if sucessful_movement {
                size += 1;
                remaining_moves = node
                    .with_sprite_mut(sprite_key, |sprite| {
                        sprite.took_a_move();
                        sprite.moves()
                    })
                    .unwrap();
            }
            // Tap if there are no remaining moves or actions
            if remaining_moves == 0
                && node
                    .with_sprite(sprite_key, |sprite| sprite.actions().is_empty())
                    .unwrap()
            {
                node.deactivate_sprite();
                break;
            }
        }
        node.grid_mut()
            .pop_back_n(sprite_key, size.checked_sub(max_size).unwrap_or(0));

        Ok(remaining_moves)
    }

    pub fn deactivate_sprite(&mut self) -> bool {
        self.node
            .as_mut()
            .map(|node| {
                node.deactivate_sprite();
                true
            })
            .unwrap_or(false)
    }

    // TODO use logic in apply action, and make this create the game action?
    pub fn activate_sprite(&mut self, sprite_key: usize) -> bool {
        if let Some(node) = self.node.as_mut() {
            node.activate_sprite(sprite_key)
        } else {
            false
        }
    }

    pub fn active_sprite_key(&self) -> Option<usize> {
        self.node().and_then(|node| node.active_sprite_key())
    }

    pub fn from(node: Option<Node>) -> Self {
        GameState {
            node,
            _world_map: WorldMap { nodes: 1 },
        }
    }

    // TODO look at this
    pub fn waiting_on_player_input(&self) -> bool {
        true
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

impl GameAction {
    pub fn next() -> GameAction {
        GameAction::Next
    }
}
