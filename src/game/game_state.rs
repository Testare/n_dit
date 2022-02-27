use super::{Animation, Direction, Node, Point, Team, WorldMap};

#[derive(Debug)]
pub struct GameState {
    _world_map: WorldMap,
    node: Option<Node>,
    animation: Option<Animation>,
}

impl GameState {
    pub fn animation(&self) -> Option<&Animation> {
        self.animation.as_ref()
    }

    pub fn node(&self) -> Option<&Node> {
        self.node.as_ref()
    }

    pub(super) fn to_mutable_pieces<'a>(&'a mut self) -> (&'a mut Option<Node>, &'a mut Option<Animation>) {
        let GameState{node, animation, ..} = self;
        (node, animation)
    }

    pub fn node_mut(&mut self) -> Option<&mut Node> {
        self.node.as_mut()
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
    // Or perhaps just remove this entirely
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

    // From trait?
    pub fn from(node: Option<Node>) -> Self {
        GameState {
            node,
            _world_map: WorldMap { nodes: 1 },
            animation: None,
        }
    }

    pub fn waiting_on_player_input(&self) -> bool {
    // TODO Check if there is an active animation?
        self.node()
            .map(|node| node.active_team() == Team::PlayerTeam)
            .unwrap_or(false)
    }

    pub fn set_animation<A: Into<Option<Animation>>>(&mut self, animation: A) {
        self.animation = animation.into();
    }

    pub fn apply_action(&mut self, game_action: &GameAction) -> Result<(), String> {
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
                        if node.activate_sprite(*sprite_key) {
                            Ok(())
                        } else {
                            Err(String::from("Sprite does not exist"))
                        }
                    }),
                _ => unimplemented!("TODO other game actions"),
            }
        } else {
            if let GameAction::Next = game_action {
                // Check lose conditions for Node
                Animation::next(self)
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
