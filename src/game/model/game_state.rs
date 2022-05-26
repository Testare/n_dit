mod game_change;

pub use game_change::GameChange;

use super::super::{Direction, Inventory, Node, Point, Team, WorldMap};
use super::animation::Animation;
use log::debug;

#[derive(Debug)]
pub struct GameState {
    _world_map: WorldMap,
    node: Option<Node>,
    animation: Option<Animation>,
    inventory: Inventory,
}

impl GameState {
    pub fn player_mon(&self) -> usize {
        self.inventory.wallet()
    }

    fn state_check_after_player_action(&mut self) {
        if let Some(node) = self.node_mut() {
            let enemy_sprites_remaining = node.sprite_keys_for_team(Team::EnemyTeam).len();
            if enemy_sprites_remaining == 0 {
                panic!("No enemies remain! You win!")
            }
            // if node.active_team() == Team::PlayerTeam {
            let untapped_player_sprites_remaining = node
                .filtered_sprite_keys(|_, sprite| {
                    sprite.team() == node.active_team() && !sprite.tapped()
                })
                .len();

            if untapped_player_sprites_remaining == 0 {
                node.change_active_team();
            }
        }
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

    pub fn active_sprite_key(&self) -> Option<usize> {
        self.node().and_then(|node| node.active_sprite_key())
    }

    // From trait?
    pub fn from(node: Option<Node>) -> Self {
        GameState {
            node,
            _world_map: WorldMap { nodes: 1 },
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

    // This should be the only method that takes a mutable reference and
    // is public outside of the game module
    pub(super) fn apply_action(&mut self, game_action: &GameAction) -> Result<(), String> {
        debug!("Game action called: {:#?}", game_action);
        match game_action {
            GameAction::ActivateSprite(sprite_key) => {
                if self.node_action(|node| node.activate_sprite(*sprite_key))? {
                    Ok(())
                } else {
                    Err("Sprite does not exist".to_string())
                }
            }
            GameAction::DeactivateSprite => {
                self.node_action(|node| {
                    node.deactivate_sprite();
                })?;
                self.state_check_after_player_action();
                Ok(())
            }
            GameAction::TakeSpriteAction(action_index, pt) => {
                self.node_action(|node| {
                    node.perform_sprite_action(*action_index, *pt);
                })?;
                self.state_check_after_player_action();
                Ok(())
            }
            GameAction::MoveActiveSprite(directions) => {
                let pickups = self.node_action(|node| node.move_active_sprite(directions))??;
                for pickup in pickups {
                    self.inventory.pick_up(pickup);
                }
                Ok(())
            }
            GameAction::Next => Animation::next(self),
        }
    }

    fn node_action<R, F: FnOnce(&mut Node) -> R>(&mut self, action: F) -> Result<R, String> {
        self.node
            .as_mut()
            .ok_or_else(|| String::from("Action doesn't make sense when we're not in a node"))
            .map(action)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum GameAction {
    Next,                  // when we're not waiting on player_input, go to next action.
    ActivateSprite(usize), // Starts using a unit.
    DeactivateSprite,      // Finishes using a unit
    MoveActiveSprite(Vec<Direction>),
    TakeSpriteAction(usize, Point),
}

impl GameAction {
    pub fn next() -> GameAction {
        GameAction::Next
    }

    pub fn activate_sprite(sprite_key: usize) -> GameAction {
        GameAction::ActivateSprite(sprite_key)
    }

    pub fn deactivate_sprite() -> GameAction {
        GameAction::DeactivateSprite
    }

    pub fn take_sprite_action(action_index: usize, pnt: Point) -> GameAction {
        GameAction::TakeSpriteAction(action_index, pnt)
    }

    pub fn move_active_sprite(directions: Vec<Direction>) -> GameAction {
        GameAction::MoveActiveSprite(directions)
    }
}
