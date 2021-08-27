use super::{Node, WorldMap};

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

    pub fn waiting_on_player_input() -> bool {
        unimplemented!("TODO: Return true when waiting for player action. False indicates that the game state should update itself without player input before the next loop")
    }
}
