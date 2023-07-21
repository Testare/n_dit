mod game_change;

pub use game_change::GameChange;
use serde::{Deserialize, Serialize};

use super::super::{Inventory, Node, Team, WorldMap};
use super::animation::Animation;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GameState {
    node: Option<Node>,
    animation: Option<Animation>,
    inventory: Inventory,
}

impl GameState {
    pub fn player_mon(&self) -> usize {
        self.node()
            .map(|node| node.inventory().wallet())
            .unwrap_or(0)
            + self.inventory.wallet()
    }

    pub fn animation(&self) -> Option<&Animation> {
        self.animation.as_ref()
    }

    pub fn node(&self) -> Option<&Node> {
        self.node.as_ref()
    }

    pub(super) fn node_mut(&mut self) -> Option<&mut Node> {
        self.node.as_mut()
    }

    pub fn active_curio_key(&self) -> Option<usize> {
        self.node().and_then(|node| node.active_curio_key())
    }

    // From trait?
    pub fn from(node: Option<Node>) -> Self {
        GameState {
            node,
            animation: None,
            inventory: Inventory::default(),
        }
    }

    pub fn waiting_on_player_input(&self) -> bool {
        self.animation.is_none()
            && self
                .node()
                .map(|node| node.active_team() == Team::PlayerTeam)
                .unwrap_or(true)
    }

    pub(super) fn set_animation<A: Into<Option<Animation>>>(&mut self, animation: A) {
        self.animation = animation.into();
    }
}
