use super::super::super::error::{ErrorMsg as _, Result};
use super::super::super::StateChange;
use crate::{Direction, GameState, Node, Pickup, Point, Team};

#[derive(Debug, Clone, Copy)]
pub enum NodeChange {
    ActivateSprite(usize), // Starts using a unit.
    DeactivateSprite,      // Finishes using a unit
    FinishTurn,
    MoveActiveSprite(Direction),
    TakeSpriteAction(usize, Point),
}

type NodeChangeResult = Result<NodeChangeMetadata>;

impl Node {
    fn check_victory_conditions(&mut self) {
        let enemy_sprites_remaining = self.sprite_keys_for_team(Team::EnemyTeam).len();
        let player_sprites_remaining = self.sprite_keys_for_team(Team::PlayerTeam).len();
        if enemy_sprites_remaining == 0 {
            panic!("No enemies remain! You win!")
        }
        if player_sprites_remaining == 0 {
            panic!("You have lost")
        }
    }

    fn finish_turn_event(&mut self) -> NodeChangeResult {
        let team = self.active_team();
        self.change_active_team();
        Ok(NodeChangeMetadata::for_team(team))
    }

    fn activate_sprite_change(&mut self, sprite_index: usize) -> NodeChangeResult {
        let metadata = NodeChangeMetadata::for_team(self.active_team())
            .with_previous_active_sprite_id(self.active_sprite_key());
        if self.activate_sprite(sprite_index) {
            Ok(metadata)
        } else {
            "Unable to activate that sprite".invalid()
        }
    }

    fn activate_sprite_undo(&mut self, metadata: &NodeChangeMetadata) -> Result<()> {
        let previous_sprite_id = metadata.previous_active_sprite_id;
        // If there was a previously active sprite when this one was activated, untap it and make it active
        if let Some(sprite_id) = previous_sprite_id {
            let sprite_exists = self.with_sprite_mut(sprite_id, |mut sprite| sprite.untap());
            if sprite_exists.is_none() {
                return format!(
                    "Undo activate sprite {}, preivously active sprite does not exist",
                    sprite_id
                )
                .fail_critical();
            }
        }
        self.set_active_sprite(previous_sprite_id);
        Ok(())
    }

    fn deactivate_sprite_change(&mut self) -> NodeChangeResult {
        if let Some(sprite_key) = self.active_sprite_key() {
            self.deactivate_sprite();
            Ok(NodeChangeMetadata::for_team(self.active_team())
                .with_previous_active_sprite_id(sprite_key))
        } else {
            "No active sprite to deactivate".invalid()
        }
    }

    fn deactivate_sprite_undo(&mut self, metadata: &NodeChangeMetadata) -> Result<()> {
        let sprite_id = metadata.expect_previous_active_sprite_id()?;
        let sprite_exists = self.with_sprite_mut(sprite_id, |mut sprite| sprite.untap());
        if sprite_exists.is_none() {
            return "Deactivate sprite does not exist".fail_critical();
        }
        self.set_active_sprite(Some(sprite_id));
        Ok(())
    }

    fn move_active_sprite(&mut self, direction: Direction) -> NodeChangeResult {
        let mut pickups = self
            .with_active_sprite_mut(|mut sprite| sprite.move_sprite(&[direction]))
            .unwrap_or_else(|| "There is no active sprite".invalid())?;
        let pickup = pickups.pop();
        // TODO add pickups to node inventory
        Ok(NodeChangeMetadata::for_team(self.active_team()).with_pickup(pickup))
    }

    fn take_sprite_action_event(
        &mut self,
        sprite_action_index: usize,
        pt: Point,
    ) -> NodeChangeResult {
        let active_sprite_key = self
            .active_sprite_key()
            .ok_or_else(|| "invalid_state".invalid_msg())?;
        let action = self
            .with_sprite(active_sprite_key, |sprite| {
                sprite
                    .actions()
                    .get(sprite_action_index)
                    .map(|action| action.unwrap())
                    .ok_or_else(|| {
                        format!("Cannot find action {} in sprite", sprite_action_index)
                            .invalid_msg()
                    })
            })
            .ok_or_else(|| "Active sprite key is not an actual sprite".fail_critical_msg())??;
        action.apply(self, active_sprite_key, pt)?;
        self.deactivate_sprite();
        self.check_victory_conditions();
        Ok(NodeChangeMetadata::for_team(self.active_team()))
    }
}

impl StateChange for NodeChange {
    type Metadata = NodeChangeMetadata;
    type State = Node;

    const STATE_NAME: &'static str = "NODE";

    fn apply(&self, node: &mut Self::State) -> Result<Self::Metadata> {
        use NodeChange::*;
        match self {
            ActivateSprite(sprite_index) => node.activate_sprite_change(*sprite_index),
            DeactivateSprite => node.deactivate_sprite_change(),
            FinishTurn => node.finish_turn_event(),
            MoveActiveSprite(dir) => node.move_active_sprite(*dir),
            TakeSpriteAction(sprite_action_index, pt) => {
                node.take_sprite_action_event(*sprite_action_index, *pt)
            }
        }
    }

    fn unapply(&self, metadata: &NodeChangeMetadata, node: &mut Self::State) -> Result<()> {
        use NodeChange::*;
        match self {
            ActivateSprite(sprite_id) => {
                node.activate_sprite_undo(metadata)?;
            }
            DeactivateSprite => {
                node.deactivate_sprite_undo(metadata)?;
            }
            _ => {}
        }
        Ok(())
    }

    fn is_durable(&self, metadata: &NodeChangeMetadata) -> bool {
        use NodeChange::*;
        if metadata.team.is_ai() {
            return matches!(self, FinishTurn); // Finish turn is the only durable event
        }
        match self {
            DeactivateSprite | FinishTurn | TakeSpriteAction(_, _) => true,
            ActivateSprite(_) | MoveActiveSprite(_) => false,
        }
    }

    fn state_from_game_state(state: &mut GameState) -> Option<&mut Self::State> {
        state.node_mut()
    }
}

#[derive(Debug, Clone)]
pub struct NodeChangeMetadata {
    /// Movement or action caused these squares to drop.
    /// We should do testing to make sure they are recorded in the order of ebing dropped off and res
    dropped_squares: Vec<Point>,
    // An item was picked up during movement
    pickup: Option<Pickup>,
    previous_active_sprite_id: Option<usize>,
    team: Team,
}

impl NodeChangeMetadata {
    fn for_team(team: Team) -> NodeChangeMetadata {
        NodeChangeMetadata {
            team,
            pickup: None,
            dropped_squares: Vec::new(),
            previous_active_sprite_id: None,
        }
    }

    fn expect_previous_active_sprite_id(&self) -> Result<usize> {
        self.previous_active_sprite_id.ok_or_else(|| {
            "Missing metadata field previous_active_sprite_id required for undo".fail_critical_msg()
        })
    }

    fn with_previous_active_sprite_id<S: Into<Option<usize>>>(
        mut self,
        sprite_id: S,
    ) -> NodeChangeMetadata {
        self.previous_active_sprite_id = sprite_id.into();
        self
    }

    fn with_pickup<P: Into<Option<Pickup>>>(mut self, pickup: P) -> NodeChangeMetadata {
        self.pickup = pickup.into();
        self
    }

    fn with_dropped_squares(mut self, dropped_squares: Vec<Point>) -> NodeChangeMetadata {
        self.dropped_squares = dropped_squares;
        self
    }
}

impl NodeChange {
    /// Helper method so StateChange trait doesn't have to be imported
    pub fn apply(&self, node: &mut Node) -> Result<NodeChangeMetadata> {
        <Self as StateChange>::apply(self, node)
    }
}
