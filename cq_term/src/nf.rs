use game_core::board::BoardPiece;
use game_core::node::{ForNode, NodeId};
use game_core::player::{ForPlayer, Player};
use game_core::quest::QuestStatus;

use crate::animation::AnimationPlayer;
use crate::board_ui::BoardPieceUi;
use crate::layout::VisibilityTty;
use crate::prelude::*;

#[derive(Debug)]
pub struct NfPlugin;

impl Plugin for NfPlugin {
    fn build(&self, app: &mut App) {
        // Might need to tweak scheduling to make sure it is added same frame.
        // I mean, rendering should be in/oafter PostUpdate anyways.
        // Needs to be after default sprites added, and an apply_deferred, but before rendering occurs
        app.add_systems(PostUpdate, sys_apply_ui_to_node_nodes);
    }
}

#[derive(Component, Debug, Default, Deref, DerefMut)]
pub struct RequiredNodes(pub Vec<NodeId>);

#[derive(Component, Debug)]
pub struct NFNode;

#[derive(Component, Debug)]
struct NFNodeUi;

// Needs to happen after board_uis have been created and sprites added
// Check new board_uis if they point to a node
fn sys_apply_ui_to_node_nodes(
    mut commands: Commands,
    board_pieces: Query<
        (Option<AsDeref<ForNode>>, Option<AsDeref<RequiredNodes>>),
        (With<BoardPiece>, With<NFNode>),
    >,
    players: Query<&QuestStatus, With<Player>>,
    mut new_board_uis: Query<
        (
            Entity,
            AsDerefCopied<BoardPieceUi>,
            &mut AnimationPlayer,
            AsDerefCopied<ForPlayer>,
        ),
        Or<(Added<BoardPieceUi>, Added<AnimationPlayer>)>,
    >,
) {
    for (bp_ui_id, bp_id, mut ap, for_player) in new_board_uis.iter_mut() {
        if let Ok((for_node, required_nodes)) = board_pieces.get(bp_id) {
            get_assert!(for_player, players, |quest_status| {
                let met_requirements = required_nodes
                    .map(|req_nodes| {
                        req_nodes
                            .iter()
                            .all(|node_id| quest_status.is_node_done(node_id))
                    })
                    .unwrap_or(true);
                commands
                    .entity(bp_ui_id)
                    .insert((VisibilityTty(met_requirements), NFNodeUi));
                if for_node
                    .map(|for_node| quest_status.is_node_done(for_node))
                    .unwrap_or(false)
                {
                    ap.set_timing(1.0);
                } else {
                    ap.set_timing(0.0);
                }
                Some(())
            });
            log::debug!("FOR PLAYER: {:?}", for_player);
        }
    }
}
