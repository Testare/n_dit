use super::super::super::error::{Error, Result};
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

    fn activate_sprite_event(&mut self, sprite_index: usize) -> NodeChangeResult {
        if self.activate_sprite(sprite_index) {
            Ok(NodeChangeMetadata::for_team(self.active_team()))
        } else {
            Err(Error::NotPossibleForState(
                "Unable to activate that sprite".to_string(),
            ))
        }
    }

    fn deactivate_sprite_event(&mut self) -> NodeChangeResult {
        self.deactivate_sprite();
        Ok(NodeChangeMetadata::for_team(self.active_team()))
    }

    fn move_active_sprite_event(&mut self, direction: Direction) -> NodeChangeResult {
        let mut pickups: Vec<Pickup> = self
            .move_active_sprite(&[direction])
            .map_err(Error::NotPossibleForState)?;
        let pickup = pickups.pop();
        // TODO add pickups to node inventory
        Ok(NodeChangeMetadata::for_team(self.active_team()).with_pickup(pickup))
    }

    fn take_sprite_action_event(
        &mut self,
        sprite_action_index: usize,
        pt: Point,
    ) -> NodeChangeResult {
        self.perform_sprite_action(sprite_action_index, pt);
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
            ActivateSprite(sprite_index) => node.activate_sprite_event(*sprite_index),
            DeactivateSprite => node.deactivate_sprite_event(),
            FinishTurn => node.finish_turn_event(),
            MoveActiveSprite(dir) => node.move_active_sprite_event(*dir),
            TakeSpriteAction(sprite_action_index, pt) => {
                node.take_sprite_action_event(*sprite_action_index, *pt)
            }
        }
    }

    fn is_durable(&self, metadata: NodeChangeMetadata) -> bool {
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
    team: Team,
}

impl NodeChangeMetadata {
    fn for_team(team: Team) -> NodeChangeMetadata {
        NodeChangeMetadata {
            team,
            pickup: None,
            dropped_squares: Vec::new(),
        }
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
