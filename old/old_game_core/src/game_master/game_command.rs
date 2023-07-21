use serde::{Deserialize, Serialize};

use super::super::error::Result;
use super::super::{GameChange, NodeChange};
use super::AuthorityGameMaster;
use crate::{Direction, Point};
/**
 * These commands are to be the sole method outside of the game core crate
 * of changing the internal state.
 *
 * For this reason it is marked as non_exhaustive, as new commands might
 * be added in the future, including new versions of the command.
 *
 * In the future we might introduce command versioning, so that different
 * implementations of commands can be implemented safely.
 *
 * Note that once we have a stable release, commands should not be
 * removed from this enum. Rather, we can mark them deprecated, and
 * eventually stop supporting them in later versions.
 *
 * This should definitely be refactored out to its own module.
 */
#[non_exhaustive]
#[derive(Clone, Eq, PartialEq, Debug, Deserialize, Serialize)]
pub enum GameCommand {
    Start,    // When a player joins a server, it should execute this command
    ShutDown, // To close the server down, run this one
    Drop,     // When a player disconnects from the server, run this one.
    Next,
    Skip,
    Undo,
    InterfaceEdition(usize),
    NodeMoveActiveCurio(Direction),
    NodeActivateCurio {
        curio_id: usize, // TODO Enum for usize, name, or point
    },
    NodeDeactivateCurio,
    NodeTakeAction {
        action_name: String, // TODO Enum for usize, or name
        target: Point,
    },
    NodePlayCard {
        card_name: String, // TODO Need a way to deliniate between multiple cards of the same name
        target_access_point: Point, // TODO Enum for piece ID or point
    },
    NodeReadyToPlay,
}

pub(super) fn apply_command_dispatch(
    gm: &mut AuthorityGameMaster,
    command: &GameCommand,
) -> Result<()> {
    use GameCommand::*;
    match command {
        Next => {
            if let Some(rx) = &gm.ai_action_receiver {
                let change = rx.recv().unwrap();
                let result = gm.apply(change);
                gm.check_to_run_ai(); // If we changed turns, delete the AI.
                result
            } else {
                // gm.apply(GameChange::NextPage)
                Ok(())
            }
        },
        NodeActivateCurio { curio_id } => gm.apply(NodeChange::ActivateCurio(*curio_id)),
        NodeDeactivateCurio => {
            gm.apply(NodeChange::DeactivateCurio)?;
            node_check_turn_end(gm)
        },
        NodeMoveActiveCurio(dir) => {
            gm.apply(NodeChange::MoveActiveCurio(*dir))?;
            node_check_turn_end(gm)
        },
        NodePlayCard {
            card_name,
            target_access_point,
        } => gm.apply(NodeChange::PlayCard(
            card_name.clone(),
            *target_access_point,
        )),
        NodeReadyToPlay => gm.apply(NodeChange::ReadyToPlay),
        NodeTakeAction {
            action_name,
            target,
        } => {
            gm.apply(NodeChange::TakeCurioAction(action_name.clone(), *target))?;
            node_check_turn_end(gm)
        },
        Skip => {
            unimplemented!("Skip action not yet implemented");
        },
        Undo => gm.undo_until_last_durable_event(),
        Start => Ok(()),
        _ => {
            unimplemented!("Many actions not yet implemented, such as {:?}", command);
        },
    }
}

fn node_check_turn_end(gm: &mut AuthorityGameMaster) -> Result<()> {
    let node = gm
        .state
        .node()
        .expect("How could this not exist if DeactivateCurio successful?");

    if node.untapped_curios_on_active_team() == 0 {
        // TODO Configurable
        gm.apply(NodeChange::FinishTurn)?;
        gm.check_to_run_ai();
    }
    Ok(())
}
