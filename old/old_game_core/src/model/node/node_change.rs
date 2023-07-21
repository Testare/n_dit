use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::super::super::error::{ErrorMsg as _, Result};
use super::super::super::{Metadata, StateChange};
use super::super::inventory::CardId;
pub use super::super::keys::node_change_keys as keys;
use super::Sprite;
use crate::{Curio, Direction, GameState, Node, Point, Team};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NodeChange {
    ActivateCurio(usize), // Starts using a unit.
    DeactivateCurio,      // Finishes using a unit
    FinishTurn,
    MoveActiveCurio(Direction),
    TakeCurioAction(String, Point),
    PlayCard(String, Point),
    ReadyToPlay,
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
        if !self.table_set {
            return "Table hasn't been set yet".invalid();
        }
        let metadata = self.default_metadata()?;
        self.change_active_team();
        Ok(metadata)
    }

    fn finish_turn_undo(&mut self, metadata: &Metadata) -> Result<()> {
        self.set_active_team(metadata.expect(keys::TEAM)?);
        Ok(())
    }

    fn activate_curio_change(&mut self, curio_index: usize) -> NodeChangeResult {
        if !self.table_set {
            return "Table hasn't been set yet".invalid();
        }
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
        if !self.table_set {
            return "Table hasn't been set yet".invalid();
        }
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
        if !self.table_set {
            return "Table hasn't been set yet".invalid();
        }
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

    fn take_curio_action_change(&mut self, action_name: &str, pt: Point) -> NodeChangeResult {
        if !self.table_set {
            return "Table hasn't been set yet".invalid();
        }
        let active_curio_key = self
            .active_curio_key()
            .ok_or_else(|| "No active curio".invalid_msg())?;
        let mut metadata = self.default_metadata()?;
        let action = self
            .with_curio(active_curio_key, |curio| {
                curio.action(action_name).ok_or_else(|| {
                    format!("Cannot find action {} in curio", action_name).invalid_msg()
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
        action_name: &str,
        target: Point,
        metadata: &Metadata,
    ) -> Result<()> {
        let active_curio_key = metadata.expect(keys::PERFORMING_CURIO)?;
        let action = self
            .action_dictionary
            .get(action_name)
            .ok_or_else(|| "Action not found".fail_critical_msg())?;
        self.set_active_curio(Some(active_curio_key));
        let curio_action_metadata = metadata.expect(keys::CURIO_ACTION_METADATA)?;
        action.unapply(self, active_curio_key, target, &curio_action_metadata)?;
        let curio_not_found = self
            .with_curio_mut(active_curio_key, |mut curio| curio.untap())
            .is_none();
        if curio_not_found {
            "Take curio action curio does not exist".fail_critical()
        } else {
            Ok(())
        }
    }

    fn play_card_change(&mut self, card_name: &str, pt: Point) -> NodeChangeResult {
        if self.table_set {
            return "Table is already set".invalid();
        }
        let mut metadata = self.default_metadata()?;
        let card_id = self
            .inventory
            .card_id(card_name)
            .ok_or_else(|| format!("Could not find card [{}]", card_name).invalid_msg())?;
        let Node {
            grid, inventory, ..
        } = self;
        match grid.item_at_mut(pt) {
            Some(Sprite::AccessPoint(crd)) => {
                // OLD CARD
                if let Some(old_card_id) = crd {
                    if *old_card_id == card_id {
                        format!("Card {} is already played there", card_name).invalid()?
                    }
                    // Return card
                    metadata.put(keys::REPLACED_CARD, old_card_id)?;
                    inventory.deck_mut().return_card(old_card_id)?;
                }
                inventory.deck_mut().play_card(&card_id)?;
                let _ = crd.insert(card_id);
            },
            _ => format!("No access point at pt [{:?}]", pt).invalid()?,
        }

        Ok(metadata)
    }

    fn play_card_undo(&mut self, pt: Point, metadata: &Metadata) -> Result<()> {
        let old_card_id = metadata.get(keys::REPLACED_CARD)?;
        let Node {
            grid, inventory, ..
        } = self;
        match grid.item_at_mut(pt) {
            Some(Sprite::AccessPoint(crd)) => {
                if let Some(played_card) = crd {
                    inventory.deck_mut().return_card(played_card)
                } else {
                    "Error during undo, no card was actually played".fail_critical()
                }?;
                if let Some(old_card) = &old_card_id {
                    inventory.deck_mut().play_card(old_card)?;
                }
                *crd = old_card_id;
            },
            _ => format!("No access point at pt [{:?}] to undo", pt).fail_critical()?,
        }
        Ok(())
    }

    fn ready_to_play_change(&mut self) -> NodeChangeResult {
        if self.table_set {
            return "Table is already set".invalid();
        }
        let mut metadata = Metadata::default();
        let access_point_map: Result<HashMap<usize, (Point, Option<CardId>)>> =
            self.grid()
                .filtered_keys(|_, sprite| matches!(sprite, Sprite::AccessPoint(_)))
                .into_iter()
                .map(|access_point_key| {
                    let pt: Point = self.grid.head(access_point_key).unwrap();

                    if let Some(Sprite::AccessPoint(crd_opt)) = self.grid().item(access_point_key) {
                        let result = (access_point_key, (pt, crd_opt.clone()));
                        if let Some(card_id) = crd_opt {
                            // TODO Definitely need some code cleanup over here
                            let card =
                                self.inventory.deck().card_by_id(card_id).ok_or_else(|| {
                                    "Can't find card in deck".fail_reversible_msg()
                                })?;

                            let card_def = self
                                .card_dictionary
                                .get(card.basis())
                                .ok_or_else(|| "Card not found in assets".fail_reversible_msg())?;

                            let mut builder = Curio::builder();
                            let builder = builder
                                .team(Team::PlayerTeam)
                                .metadata(card.metadata.clone())
                                .actions(&card_def.actions)
                                .speed(card_def.speed)
                                .max_size(card_def.max_size)
                                .display(card_def.display.clone())
                                .name(
                                    card.nickname
                                        .as_ref()
                                        .unwrap_or_else(|| card.basis())
                                        .clone(),
                                );

                            self.grid_mut().replace_item(
                                access_point_key,
                                Sprite::Curio(builder.build().ok_or_else(|| {
                                    "Failed to convert asset to curio".fail_critical_msg()
                                })?),
                            );
                        } else {
                            self.grid_mut().pop_back(access_point_key);
                        }
                        Ok(result)
                    } else {
                        "Some crazy access point failure".fail_critical()
                    }
                })
                .collect();

        metadata.put(keys::ACCESS_POINT_MAP, &access_point_map?)?;
        self.table_set = true;

        // Only do if table is not yet set
        // Find all access points and card ids
        // Make a map and add it to metadata
        // Transform access points to curios
        // Mark the table as set
        Ok(metadata)
    }

    fn ready_to_play_undo(&mut self, metadata: &Metadata) -> Result<()> {
        let access_point_map = metadata.expect(keys::ACCESS_POINT_MAP)?;
        for (key, (pt, crd_opt)) in access_point_map.into_iter() {
            if crd_opt.is_some() {
                self.grid_mut()
                    .replace_item(key, Sprite::AccessPoint(crd_opt));
            } else if self.grid().item(key).is_none() {
                unsafe {
                    // Key checked to not exist
                    self.grid_mut()
                        .return_item_with_key(key, pt, Sprite::AccessPoint(None));
                }
            } else {
                "Undo failed: somehow an empty access point became a sprite".fail_critical()?;
            }
        }

        self.table_set = false;
        Ok(())
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
            TakeCurioAction(action_name, pt) => {
                node.take_curio_action_change(action_name.as_str(), *pt)
            },
            PlayCard(card_name, pt) => node.play_card_change(card_name.as_str(), *pt),
            ReadyToPlay => node.ready_to_play_change(),
            _ => {
                unimplemented!("Unimplemented NodeChange")
            },
        }
    }

    fn unapply(&self, metadata: &Metadata, node: &mut Self::State) -> Result<()> {
        use NodeChange::*;

        match self {
            ActivateCurio(_) => node.activate_curio_undo(metadata),
            DeactivateCurio => node.deactivate_curio_undo(metadata),
            FinishTurn => node.finish_turn_undo(metadata),
            MoveActiveCurio(_) => node.move_active_curio_undo(metadata),
            TakeCurioAction(action_name, target) => {
                node.take_curio_action_undo(action_name.as_str(), *target, metadata)
            },
            PlayCard(_, pt) => node.play_card_undo(*pt, metadata),
            ReadyToPlay => node.ready_to_play_undo(metadata),
            _ => {
                unimplemented!("Unimplemented NodeChange")
            },
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
            DeactivateCurio | FinishTurn | TakeCurioAction(_, _) | PlayCard(_, _) | ReadyToPlay => {
                true
            },
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
