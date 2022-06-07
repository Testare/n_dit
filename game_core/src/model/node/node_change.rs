use serde::{Deserialize, Serialize};

use super::super::super::error::{ErrorMsg as _, Result};
use super::super::super::{Metadata, StateChange};
pub use super::super::keys::node_change_keys as keys;
use super::Sprite;
use crate::{Direction, GameState, Node, Point, Team};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum NodeChange {
    ActivateCurio(usize), // Starts using a unit.
    DeactivateCurio,      // Finishes using a unit
    FinishTurn,
    MoveActiveCurio(Direction),
    TakeCurioAction(usize, Point),
}

type NodeChangeResult = Result<Metadata>;

impl Node {
    fn check_victory_conditions(&mut self) {
        let enemy_curios_remaining = self.curio_keys_for_team(Team::EnemyTeam).len();
        let player_curios_remaining = self.curio_keys_for_team(Team::PlayerTeam).len();
        if enemy_curios_remaining == 0 {
            panic!("No enemies remain! You win!")
        }
        if player_curios_remaining == 0 {
            panic!("You have lost")
        }
    }

    fn finish_turn_change(&mut self) -> NodeChangeResult {
        let metadata = self.default_metadata()?;
        self.change_active_team();
        Ok(metadata)
    }

    fn finish_turn_undo(&mut self, metadata: &Metadata) -> Result<()> {
        self.set_active_team(metadata.expect(keys::TEAM)?);
        Ok(())
    }

    fn activate_curio_change(&mut self, curio_index: usize) -> NodeChangeResult {
        let metadata = self.default_metadata()?;
        if self.activate_curio(curio_index) {
            Ok(metadata)
        } else {
            "Unable to activate that curio".invalid()
        }
    }

    fn activate_curio_undo(&mut self, metadata: &Metadata) -> Result<()> {
        let previous_curio_id = metadata.get(keys::PERFORMING_CURIO)?;
        // If there was a previously active curio when this one was activated, untap it and make it active
        if let Some(curio_id) = previous_curio_id {
            let curio_exists = self.with_curio_mut(curio_id, |mut curio| curio.untap());
            if curio_exists.is_none() {
                return format!(
                    "Undo activate curio {}, preivously active curio does not exist",
                    curio_id
                )
                .fail_critical();
            }
        }
        self.set_active_curio(previous_curio_id);
        Ok(())
    }

    fn deactivate_curio_change(&mut self) -> NodeChangeResult {
        let metadata = self.default_metadata()?;
        if self.active_curio_key().is_some() {
            self.deactivate_curio();
            Ok(metadata)
        } else {
            "No active curio to deactivate".invalid()
        }
    }

    fn deactivate_curio_undo(&mut self, metadata: &Metadata) -> Result<()> {
        let curio_id = metadata.expect(keys::PERFORMING_CURIO)?;
        let curio_not_found = self
            .with_curio_mut(curio_id, |mut curio| curio.untap())
            .is_none();
        if curio_not_found {
            return "Deactivate curio does not exist".fail_critical();
        }
        self.set_active_curio(Some(curio_id));
        Ok(())
    }

    fn move_active_curio(&mut self, direction: Direction) -> NodeChangeResult {
        self.with_active_curio_mut(|mut curio| curio.go(direction))
            .unwrap_or_else(|| "No active curio".invalid())
        // TODO Include standard metadata fields
    }

    fn move_active_curio_undo(&mut self, metadata: &Metadata) -> Result<()> {
        let head_pt = self
            .with_active_curio_mut(|mut curio| {
                let head_pt = curio.head();
                if let Some(pt) = metadata.get(keys::DROPPED_POINT)? {
                    curio.grow_back(pt);
                }
                curio.drop_front();
                Ok(head_pt)
            })
            .unwrap_or_else(|| "No active curio".fail_critical())?;

        if let Some(pickup) = metadata.get(keys::PICKUP)? {
            self.inventory.remove(&pickup);
            self.grid_mut()
                .put_item(head_pt, Sprite::Pickup(pickup.clone()));
        }
        Ok(())
    }

    fn take_curio_action_change(
        &mut self,
        curio_action_index: usize,
        pt: Point,
    ) -> NodeChangeResult {
        let active_curio_key = self
            .active_curio_key()
            .ok_or_else(|| "No active curio".invalid_msg())?;
        let mut metadata = self.default_metadata()?;
        let action = self
            .with_curio(active_curio_key, |curio| {
                curio.indexed_action(curio_action_index).ok_or_else(|| {
                    format!("Cannot find action {} in curio", curio_action_index).invalid_msg()
                })
            })
            .ok_or_else(|| "Active curio key is not an actual curio".fail_critical_msg())??;
        let sprite_action_metadata = action.apply(self, active_curio_key, pt)?;
        metadata.put(keys::CURIO_ACTION_METADATA, &sprite_action_metadata)?;
        self.deactivate_curio();
        self.check_victory_conditions();
        Ok(metadata)
    }

    fn take_curio_action_undo(
        &mut self,
        action_index: usize,
        target: Point,
        metadata: &Metadata,
    ) -> Result<()> {
        let active_curio_key = metadata.expect(keys::PERFORMING_CURIO)?;
        let curio_not_found = self
            .with_curio_mut(active_curio_key, |mut curio| curio.untap())
            .is_none();
        if curio_not_found {
            return "Take curio action curio does not exist".fail_critical();
        }
        let action = self
            .with_curio(active_curio_key, |curio| {
                curio.indexed_action(action_index).ok_or_else(|| {
                    format!("Cannot find action {} in curio", action_index).fail_critical_msg()
                })
            })
            .ok_or_else(|| "Active curio key is not an actual curio".fail_critical_msg())??;
        // TODO This logic will likely be more complex

        self.set_active_curio(Some(active_curio_key));
        let curio_action_metadata = metadata.expect(keys::CURIO_ACTION_METADATA)?;
        action.unapply(self, active_curio_key, target, &curio_action_metadata)
    }
}

impl StateChange for NodeChange {
    type Metadata = Metadata;
    type State = Node;

    const STATE_NAME: &'static str = "NODE";

    fn apply(&self, node: &mut Self::State) -> Result<Self::Metadata> {
        use NodeChange::*;
        match self {
            ActivateCurio(curio_index) => node.activate_curio_change(*curio_index),
            DeactivateCurio => node.deactivate_curio_change(),
            FinishTurn => node.finish_turn_change(),
            MoveActiveCurio(dir) => node.move_active_curio(*dir),
            TakeCurioAction(curio_action_index, pt) => {
                node.take_curio_action_change(*curio_action_index, *pt)
            }
        }
    }

    fn unapply(&self, metadata: &Metadata, node: &mut Self::State) -> Result<()> {
        use NodeChange::*;

        match self {
            ActivateCurio(_) => node.activate_curio_undo(metadata),
            DeactivateCurio => node.deactivate_curio_undo(metadata),
            FinishTurn => node.finish_turn_undo(metadata),
            MoveActiveCurio(_) => node.move_active_curio_undo(metadata),
            TakeCurioAction(action_index, target) => {
                node.take_curio_action_undo(*action_index, *target, metadata)
            }
        }
    }

    fn is_durable(&self, metadata: &Metadata) -> bool {
        use NodeChange::*;
        if metadata
            .get(keys::TEAM)
            .unwrap_or(None)
            .map(|team| team.is_ai())
            .unwrap_or(false)
        {
            return matches!(self, FinishTurn); // Finish turn is the only durable event
        }
        match self {
            DeactivateCurio | FinishTurn | TakeCurioAction(_, _) => true,
            ActivateCurio(_) | MoveActiveCurio(_) => false,
        }
    }

    fn state_from_game_state(state: &mut GameState) -> Option<&mut Self::State> {
        state.node_mut()
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct SpritePoint(pub usize, pub Point);

impl NodeChange {
    /// Helper method so StateChange trait doesn't have to be imported
    pub fn apply(&self, node: &mut Node) -> Result<Metadata> {
        <Self as StateChange>::apply(self, node)
    }
}
