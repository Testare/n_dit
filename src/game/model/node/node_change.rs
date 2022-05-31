use super::super::super::error::{ErrorMsg as _, Result};
use super::super::super::StateChange;
use super::Piece;
use crate::{Direction, GameState, Node, Pickup, Point, Team};
use getset::Getters;

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

    fn finish_turn_change(&mut self) -> NodeChangeResult {
        let team = self.active_team();
        self.change_active_team();
        Ok(NodeChangeMetadata::for_team(team))
    }

    fn finish_turn_undo(&mut self, metadata: &NodeChangeMetadata) -> Result<()> {
        self.set_active_team(metadata.team);
        Ok(())
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
        let sprite_not_found = self
            .with_sprite_mut(sprite_id, |mut sprite| sprite.untap())
            .is_none();
        if sprite_not_found {
            return "Deactivate sprite does not exist".fail_critical();
        }
        self.set_active_sprite(Some(sprite_id));
        Ok(())
    }

    fn move_active_sprite(&mut self, direction: Direction) -> NodeChangeResult {
        self.with_active_sprite_mut(|mut sprite| sprite.go(direction))
            .unwrap_or_else(|| "No active sprite".invalid())
    }

    fn move_active_sprite_undo(&mut self, metadata: &NodeChangeMetadata) -> Result<()> {
        let head_pt = self
            .with_active_sprite_mut(|mut sprite| {
                let head_pt = sprite.head();
                for dropped_square in metadata.dropped_squares() {
                    if dropped_square.0 == sprite.key() {
                        sprite.grow_back(dropped_square.1);
                    }
                }
                sprite.drop_front();
                Ok(head_pt)
            })
            .unwrap_or_else(|| "No active sprite".fail_critical())?;

        if let Some(pickup) = metadata.pickup() {
            self.inventory.remove(pickup);
            self.grid_mut()
                .put_item(head_pt, Piece::Pickup(pickup.clone()));
        }
        Ok(())
    }

    fn take_sprite_action_change(
        &mut self,
        sprite_action_index: usize,
        pt: Point,
    ) -> NodeChangeResult {
        let active_sprite_key = self
            .active_sprite_key()
            .ok_or_else(|| "No active sprite".invalid_msg())?;
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
        let metadata = action.apply(self, active_sprite_key, pt)?;
        let active_sprite_key = self.active_sprite_key();
        self.deactivate_sprite();
        self.check_victory_conditions();
        Ok(metadata.with_previous_active_sprite_id(active_sprite_key))
    }

    fn take_sprite_action_undo(
        &mut self,
        action_index: usize,
        target: Point,
        metadata: &NodeChangeMetadata,
    ) -> Result<()> {
        let active_sprite_key = metadata.expect_previous_active_sprite_id()?;
        let sprite_not_found = self
            .with_sprite_mut(active_sprite_key, |mut sprite| sprite.untap())
            .is_none();
        if sprite_not_found {
            return "Take sprite action sprite does not exist".fail_critical();
        }
        let action = self
            .with_sprite(active_sprite_key, |sprite| {
                sprite
                    .actions()
                    .get(action_index)
                    .map(|action| action.unwrap())
                    .ok_or_else(|| {
                        format!("Cannot find action {} in sprite", action_index).fail_critical_msg()
                    })
            })
            .ok_or_else(|| "Active sprite key is not an actual sprite".fail_critical_msg())??;
        // TODO This logic will likely be more complex

        self.set_active_sprite(Some(active_sprite_key));
        action.unapply(self, active_sprite_key, target, metadata)?;
        for dropped_square in metadata.dropped_squares() {
            self.with_sprite_mut(dropped_square.0, |mut sprite| {
                sprite.grow_back(dropped_square.1);
            })
            .ok_or_else(|| {
                format!(
                    "Could not find sprite to undo dropped square {:?}",
                    dropped_square
                )
                .fail_critical_msg()
            })?;
        }
        Ok(())
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
            FinishTurn => node.finish_turn_change(),
            MoveActiveSprite(dir) => node.move_active_sprite(*dir),
            TakeSpriteAction(sprite_action_index, pt) => {
                node.take_sprite_action_change(*sprite_action_index, *pt)
            }
        }
    }

    fn unapply(&self, metadata: &NodeChangeMetadata, node: &mut Self::State) -> Result<()> {
        use NodeChange::*;
        match self {
            ActivateSprite(_) => {
                node.activate_sprite_undo(metadata)?;
            }
            DeactivateSprite => {
                node.deactivate_sprite_undo(metadata)?;
            }
            FinishTurn => {
                node.finish_turn_undo(metadata)?;
            }
            MoveActiveSprite(_) => {
                node.move_active_sprite_undo(metadata)?;
            }
            TakeSpriteAction(action_index, target) => {
                node.take_sprite_action_undo(*action_index, *target, metadata)?;
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

#[derive(Debug, Clone, Copy)]
pub struct DroppedSquare(pub usize, pub Point);

#[derive(Debug, Clone, Getters)]
pub struct NodeChangeMetadata {
    /// Movement or action caused these squares to drop.
    /// We should do testing to make sure they are recorded in the order of ebing dropped off and res
    #[get = "pub"]
    dropped_squares: Vec<DroppedSquare>,
    // An item was picked up during movement
    #[get = "pub"]
    pickup: Option<Pickup>,
    #[get = "pub"]
    previous_active_sprite_id: Option<usize>,
    #[get = "pub"]
    team: Team,
    #[get = "pub"]
    deleted_piece: Option<(usize, Piece)>,
}

impl NodeChangeMetadata {
    pub(crate) fn for_team(team: Team) -> NodeChangeMetadata {
        NodeChangeMetadata {
            team,
            pickup: None,
            dropped_squares: Vec::new(),
            previous_active_sprite_id: None,
            deleted_piece: None,
        }
    }

    fn expect_previous_active_sprite_id(&self) -> Result<usize> {
        self.previous_active_sprite_id.ok_or_else(|| {
            "Missing metadata field previous_active_sprite_id required for undo".fail_critical_msg()
        })
    }

    pub(crate) fn with_previous_active_sprite_id<S: Into<Option<usize>>>(
        mut self,
        sprite_id: S,
    ) -> NodeChangeMetadata {
        self.previous_active_sprite_id = sprite_id.into();
        self
    }

    pub(crate) fn with_pickup<P: Into<Option<Pickup>>>(mut self, pickup: P) -> NodeChangeMetadata {
        self.pickup = pickup.into();
        self
    }

    pub(crate) fn with_dropped_squares(
        mut self,
        dropped_squares: Vec<DroppedSquare>,
    ) -> NodeChangeMetadata {
        self.dropped_squares = dropped_squares;
        self
    }

    pub(crate) fn with_deleted_piece<P: Into<Option<(usize, Piece)>>>(
        mut self,
        deleted_piece: P,
    ) -> NodeChangeMetadata {
        self.deleted_piece = deleted_piece.into();
        self
    }
}

impl NodeChange {
    /// Helper method so StateChange trait doesn't have to be imported
    pub fn apply(&self, node: &mut Node) -> Result<NodeChangeMetadata> {
        <Self as StateChange>::apply(self, node)
    }
}
