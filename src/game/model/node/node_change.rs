use getset::Getters;
use serde::{Deserialize, Serialize};

use super::super::super::error::{ErrorMsg as _, Result};
use super::super::super::{Metadata, StateChange};
use super::Piece;
use crate::{Direction, GameState, Node, Pickup, Point, Team};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum NodeChange {
    ActivateCurio(usize), // Starts using a unit.
    DeactivateCurio,      // Finishes using a unit
    FinishTurn,
    MoveActiveCurio(Direction),
    TakeCurioAction(usize, Point),
}

type NodeChangeResult = Result<NodeChangeMetadata>;

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
        let team = self.active_team();
        self.change_active_team();
        Ok(NodeChangeMetadata::for_team(team))
    }

    fn finish_turn_undo(&mut self, metadata: &NodeChangeMetadata) -> Result<()> {
        self.set_active_team(metadata.team);
        Ok(())
    }

    fn activate_curio_change(&mut self, curio_index: usize) -> NodeChangeResult {
        let metadata = NodeChangeMetadata::for_team(self.active_team())
            .with_previous_active_curio_id(self.active_curio_key());
        if self.activate_curio(curio_index) {
            Ok(metadata)
        } else {
            "Unable to activate that curio".invalid()
        }
    }

    fn activate_curio_undo(&mut self, metadata: &NodeChangeMetadata) -> Result<()> {
        let previous_curio_id = metadata.previous_active_curio_id;
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
        if let Some(curio_key) = self.active_curio_key() {
            self.deactivate_curio();
            Ok(NodeChangeMetadata::for_team(self.active_team())
                .with_previous_active_curio_id(curio_key))
        } else {
            "No active curio to deactivate".invalid()
        }
    }

    fn deactivate_curio_undo(&mut self, metadata: &NodeChangeMetadata) -> Result<()> {
        let curio_id = metadata.expect_previous_active_curio_id()?;
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
        let metadata: Metadata = self
            .with_active_curio_mut(|mut curio| curio.go(direction))
            .unwrap_or_else(|| "No active curio".invalid())?;
        NodeChangeMetadata::from(&metadata)
    }

    fn move_active_curio_undo(&mut self, metadata: &NodeChangeMetadata) -> Result<()> {
        let head_pt = self
            .with_active_curio_mut(|mut curio| {
                let head_pt = curio.head();
                for dropped_square in metadata.dropped_squares() {
                    if dropped_square.0 == curio.key() {
                        curio.grow_back(dropped_square.1);
                    }
                }
                curio.drop_front();
                Ok(head_pt)
            })
            .unwrap_or_else(|| "No active curio".fail_critical())?;

        if let Some(pickup) = metadata.pickup() {
            self.inventory.remove(pickup);
            self.grid_mut()
                .put_item(head_pt, Piece::Pickup(pickup.clone()));
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
        let action = self
            .with_curio(active_curio_key, |curio| {
                curio
                    .actions()
                    .get(curio_action_index)
                    .map(|action| action.unwrap())
                    .ok_or_else(|| {
                        format!("Cannot find action {} in curio", curio_action_index)
                            .invalid_msg()
                    })
            })
            .ok_or_else(|| "Active curio key is not an actual curio".fail_critical_msg())??;
        let metadata = NodeChangeMetadata::from(&action.apply(self, active_curio_key, pt)?)?;
        let active_curio_key = self.active_curio_key();
        self.deactivate_curio();
        self.check_victory_conditions();
        Ok(metadata.with_previous_active_curio_id(active_curio_key))
    }

    fn take_curio_action_undo(
        &mut self,
        action_index: usize,
        target: Point,
        metadata: &NodeChangeMetadata,
    ) -> Result<()> {
        let active_curio_key = metadata.expect_previous_active_curio_id()?;
        let curio_not_found = self
            .with_curio_mut(active_curio_key, |mut curio| curio.untap())
            .is_none();
        if curio_not_found {
            return "Take curio action curio does not exist".fail_critical();
        }
        let action = self
            .with_curio(active_curio_key, |curio| {
                curio
                    .actions()
                    .get(action_index)
                    .map(|action| action.unwrap())
                    .ok_or_else(|| {
                        format!("Cannot find action {} in curio", action_index).fail_critical_msg()
                    })
            })
            .ok_or_else(|| "Active curio key is not an actual curio".fail_critical_msg())??;
        // TODO This logic will likely be more complex

        self.set_active_curio(Some(active_curio_key));
        action.unapply(self, active_curio_key, target, &metadata.to_metadata()?)?;
        for dropped_square in metadata.dropped_squares() {
            self.with_curio_mut(dropped_square.0, |mut curio| {
                curio.grow_back(dropped_square.1);
            })
            .ok_or_else(|| {
                format!(
                    "Could not find curio to undo dropped square {:?}",
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
            ActivateCurio(curio_index) => node.activate_curio_change(*curio_index),
            DeactivateCurio => node.deactivate_curio_change(),
            FinishTurn => node.finish_turn_change(),
            MoveActiveCurio(dir) => node.move_active_curio(*dir),
            TakeCurioAction(curio_action_index, pt) => {
                node.take_curio_action_change(*curio_action_index, *pt)
            }
        }
    }

    fn unapply(&self, metadata: &NodeChangeMetadata, node: &mut Self::State) -> Result<()> {
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

    fn is_durable(&self, metadata: &NodeChangeMetadata) -> bool {
        use NodeChange::*;
        if metadata.team.is_ai() {
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
pub struct DroppedSquare(pub usize, pub Point);

pub mod keys {
    use super::DroppedSquare;
    use crate::{Pickup, Piece, Team};
    use typed_key::{typed_key, Key};

    pub const TEAM: Key<Team> = typed_key!("team");
    pub const PICKUP: Key<Pickup> = typed_key!("pickup");
    pub const DROPPED_SQUARES: Key<Vec<DroppedSquare>> = typed_key!("droppedSquares");
    pub const PREVIOUS_ACTIVE_CURIO: Key<usize> = typed_key!("previousActiveCurio");
    pub const DELETED_PIECE: Key<(usize, Piece)> = typed_key!("deletedPiece");
}

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
    previous_active_curio_id: Option<usize>,
    #[get = "pub"]
    team: Team,
    #[get = "pub"]
    deleted_piece: Option<(usize, Piece)>,
}

impl NodeChangeMetadata {
    /// Likely just temporary
    pub fn to_metadata(&self) -> Result<Metadata> {
        let mut metadata = Metadata::new();
        use keys::*;
        metadata.put(TEAM, &self.team)?;
        metadata.put_nonempty(DROPPED_SQUARES, &self.dropped_squares)?;
        metadata.put_optional(PICKUP, &self.pickup)?;
        metadata.put_optional(PREVIOUS_ACTIVE_CURIO, &self.previous_active_curio_id)?;
        metadata.put_optional(DELETED_PIECE, &self.deleted_piece)?;
        Ok(metadata)
    }

    /// Likely just temporary
    pub fn from(metadata: &Metadata) -> Result<NodeChangeMetadata> {
        use keys::*;
        let team = metadata.expect(TEAM)?;
        let dropped_squares = metadata.get(DROPPED_SQUARES)?.unwrap_or_default();
        let pickup = metadata.get(PICKUP)?;
        let previous_active_curio_id = metadata.get(PREVIOUS_ACTIVE_CURIO)?;
        let deleted_piece = metadata.get(DELETED_PIECE)?;
        Ok(NodeChangeMetadata {
            team,
            pickup,
            dropped_squares,
            previous_active_curio_id,
            deleted_piece,
        })
    }

    pub(crate) fn for_team(team: Team) -> NodeChangeMetadata {
        NodeChangeMetadata {
            team,
            pickup: None,
            dropped_squares: Vec::new(),
            previous_active_curio_id: None,
            deleted_piece: None,
        }
    }

    fn expect_previous_active_curio_id(&self) -> Result<usize> {
        self.previous_active_curio_id.ok_or_else(|| {
            "Missing metadata field previous_active_curio_id required for undo".fail_critical_msg()
        })
    }

    pub(crate) fn with_previous_active_curio_id<S: Into<Option<usize>>>(
        mut self,
        curio_id: S,
    ) -> NodeChangeMetadata {
        self.previous_active_curio_id = curio_id.into();
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
