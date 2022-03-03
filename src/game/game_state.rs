use super::{Animation, Direction, Node, Point, Team, WorldMap};
use log::debug;

#[derive(Debug)]
pub struct GameState {
    _world_map: WorldMap,
    node: Option<Node>,
    animation: Option<Animation>,
}

impl GameState {
    fn state_check_after_player_action(&mut self) {
        if let Some(node) = self.node_mut() {
            let enemy_sprites_remaining = node
                .filtered_sprite_keys(|_, sprite| sprite.team() == Team::EnemyTeam)
                .len();
            if enemy_sprites_remaining == 0 {
                panic!("No enemies remain! You win!")
            }

            if node.active_team() == Team::PlayerTeam {
                let untapped_player_sprites_remaining = node
                    .filtered_sprite_keys(|_, sprite| {
                        sprite.team() == Team::PlayerTeam && !sprite.tapped()
                    })
                    .len();

                if untapped_player_sprites_remaining == 0 {
                    node.change_active_team();
                    if node.active_team() == Team::EnemyTeam {
                        // This check in pla
                        let enemy_ai_actions = node.enemy_ai().generate_animation(node);
                        self.set_animation(enemy_ai_actions);
                    }
                }
            }
        }
    }

    pub fn animation(&self) -> Option<&Animation> {
        self.animation.as_ref()
    }

    pub fn node(&self) -> Option<&Node> {
        self.node.as_ref()
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
        self.animation.is_none()
    }

    pub fn set_animation<A: Into<Option<Animation>>>(&mut self, animation: A) {
        self.animation = animation.into();
    }

    pub fn apply_action(&mut self, game_action: &GameAction) -> Result<(), String> {
        if self.waiting_on_player_input() {
            debug!("Game action called: {:#?}", game_action);
            match game_action {
                GameAction::Next => Err(String::from("Waiting for player input")),
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
                GameAction::MoveActiveSprite(directions) => self.node_action(|node| {
                    node.move_active_sprite(directions)?;
                    Ok(())
                })?,
            }
        } else if let GameAction::Next = game_action {
            // TODO Check lose conditions for Node
            Animation::next(self)
        } else {
            Err(String::from(
                "Cannot accept player actions right now, next action must be 'Next'",
            ))
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
