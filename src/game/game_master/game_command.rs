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
#[derive(Clone, Eq, PartialEq, Debug)]
pub enum GameCommand {
    Next,
    Skip,
    Undo,
    InterfaceEdition(usize),
    NodeMoveActiveSprite(Direction),
    NodeActivateSprite {
        sprite_id: usize, // TODO Enum for usize, name, or point
    },
    NodeDeactivateSprite,
    NodeTakeAction {
        sprite_action_id: usize, // TODO Enum for usize, or name
        target: Point,
    },
}

pub(super) fn apply_command_dispatch(
    gm: &mut AuthorityGameMaster,
    command: &GameCommand,
) -> Result<()> {
    use GameCommand::*;
    match command {
        NodeActivateSprite { sprite_id } => gm.apply(NodeChange::ActivateSprite(*sprite_id)),
        NodeMoveActiveSprite(dir) => gm.apply(NodeChange::MoveActiveSprite(*dir)),
        NodeDeactivateSprite => {
            gm.apply(NodeChange::DeactivateSprite)?;
            let node = gm
                .state
                .node()
                .expect("How could this not exist if DeactivateSprite successful?");

            if node.untapped_sprites_on_active_team() == 0 {
                // TODO Configurable
                gm.apply(NodeChange::FinishTurn)?;
                gm.check_to_run_ai();
            }
            Ok(())
        }
        NodeTakeAction {
            sprite_action_id,
            target,
        } => {
            gm.apply(NodeChange::TakeSpriteAction(*sprite_action_id, *target))?;
            let node = gm
                .state
                .node()
                .expect("How could this not exist if TakeSpriteAction successful?");

            if node.untapped_sprites_on_active_team() == 0 {
                gm.apply(NodeChange::FinishTurn)?;
                gm.check_to_run_ai();
            }
            Ok(())
        }
        Next => {
            if let Some(rx) = &gm.ai_action_receiver {
                let change = rx.recv().unwrap();
                let result = gm.apply(change);
                gm.check_to_run_ai(); // If we changed turns, delete the AI.
                result
            } else {
                gm.apply(GameChange::NextPage)
            }
        }
        Skip => {
            unimplemented!("Skip action not yet implemented");
        }
        Undo => {
            gm.undo_until_last_durable_event()
        }
        _ => {
            unimplemented!("Many actions not yet implemented");
        }
    }
}
