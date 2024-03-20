use game_core::board::BoardPiece;
use game_core::node::{ForNode, NodeId, NodeOp};
use game_core::op::CoreOps;
use game_core::player::{ForPlayer, Player};
use game_core::quest::QuestStatus;

use crate::animation::AnimationPlayer;
use crate::base_ui::context_menu::{ContextAction, ContextActions};
use crate::board_ui::{BoardPieceUi, SelectedBoardPieceUi};
use crate::input_event::{self, MouseEventListener, MouseEventTty, MouseEventTtyKind};
use crate::layout::VisibilityTty;
use crate::prelude::*;

#[derive(Debug)]
pub struct NfPlugin;

impl Plugin for NfPlugin {
    fn build(&self, app: &mut App) {
        // Might need to tweak scheduling to make sure it is added same frame.
        // I mean, rendering should be in/oafter PostUpdate anyways.
        // Needs to be after default sprites added, and an apply_deferred, but before rendering occurs
        app.init_resource::<NfContextActions>()
            .add_systems(PostUpdate, sys_apply_ui_to_node_nodes)
            .add_systems(
                Update,
                (
                    sys_adjust_nf_ui_when_quest_status_updates,
                    mouse_network_map_nodes,
                ),
            );
    }
}

#[derive(Debug, Resource)]
pub struct NfContextActions {
    enter_node: Entity,
    enter_shop: Entity,
    select_piece: Entity,
}

impl FromWorld for NfContextActions {
    fn from_world(world: &mut World) -> Self {
        let enter_node = world
            .spawn((
                Name::new("Enter Node CA"),
                ContextAction::new("Enter Node", |id, world| {
                    (|| {
                        // try
                        let &ForPlayer(player_id) = world.get(id)?;
                        let &BoardPieceUi(bp_id) = world.get(id)?;
                        let node_sid: NodeId = world.get::<ForNode>(bp_id)?.0.clone();
                        world
                            .resource_mut::<CoreOps>()
                            .request(player_id, NodeOp::EnterNode(node_sid));
                        Some(())
                    })();
                }),
            ))
            .id();
        let enter_shop = world
            .spawn((
                Name::new("Enter Shop CA"),
                ContextAction::new("Enter Shop", |_id, _world| {
                    log::debug!("Coming soon: Shops!")
                }),
            ))
            .id();
        let select_piece = world
            .spawn((
                Name::new("Select Node CA"),
                ContextAction::new("Select Node", |id, world| {
                    (|| {
                        // try
                        let &ForPlayer(player_id) = world.get(id)?;
                        let mut selected_board_piece_ui: Mut<'_, SelectedBoardPieceUi> =
                            world.get_mut(player_id)?;
                        selected_board_piece_ui.as_deref_mut().set_if_neq(Some(id));
                        Some(())
                    })();
                }),
            ))
            .id();
        Self {
            enter_node,
            enter_shop,
            select_piece,
        }
    }
}

#[derive(Component, Debug, Default, Deref, DerefMut)]
pub struct RequiredNodes(pub Vec<NodeId>);

#[derive(Component, Debug)]
pub struct NFNode;

#[derive(Component, Debug)]
pub struct NFShop(pub String);

#[derive(Component, Debug)]
struct NFNodeUi;

// Needs to happen after board_uis have been created and sprites added
// Check new board_uis if they point to a node
fn sys_apply_ui_to_node_nodes(
    mut commands: Commands,
    res_nf_ca: Res<NfContextActions>,
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
                let mut entity_commands = commands.entity(bp_ui_id);
                entity_commands.insert((
                    VisibilityTty(met_requirements),
                    NFNodeUi,
                    MouseEventListener,
                ));
                if for_node.is_some() {
                    // TODO when show_dialogue is implemented, swap these
                    entity_commands.insert(ContextActions::new(
                        for_player,
                        &[res_nf_ca.select_piece, res_nf_ca.enter_node],
                    ));
                } else {
                    entity_commands.insert(ContextActions::new(
                        for_player,
                        &[res_nf_ca.select_piece, res_nf_ca.enter_shop],
                    ));
                }
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
        }
    }
}

// TODO should combine this system with `sys_apply_ui_to_node_nodes`
fn sys_adjust_nf_ui_when_quest_status_updates(
    players: Query<(Entity, &QuestStatus), Changed<QuestStatus>>,
    nf_nodes: Query<(Option<AsDeref<RequiredNodes>>, Option<AsDeref<ForNode>>), With<NFNode>>,
    mut nf_node_ui: Query<
        (
            AsDerefCopied<ForPlayer>,
            AsDerefCopied<BoardPieceUi>,
            &mut AnimationPlayer,
            AsDerefMut<VisibilityTty>,
        ),
        With<NFNodeUi>,
    >,
) {
    for (player_id, quest_status) in players.iter() {
        for (for_player, bp_id, mut ap, mut is_visible) in nf_node_ui.iter_mut() {
            if player_id != for_player {
                continue;
            }
            get_assert!(bp_id, nf_nodes, |(required_nodes, for_node)| {
                let met_requirements = required_nodes
                    .map(|req_nodes| {
                        req_nodes
                            .iter()
                            .all(|node_id| quest_status.is_node_done(node_id))
                    })
                    .unwrap_or(true);
                is_visible.set_if_neq(met_requirements);
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
        }
    }
}

fn mouse_network_map_nodes(
    mut evr_mouse: EventReader<MouseEventTty>,
    mut nf_nodes: Query<
        (
            AsDerefCopied<ForPlayer>,
            AsDerefCopied<BoardPieceUi>,
            &mut AnimationPlayer,
        ),
        With<NFNodeUi>,
    >,
    board_pieces: Query<Option<AsDeref<ForNode>>, (With<BoardPiece>, With<NFNode>)>,
    players: Query<(&QuestStatus,), With<Player>>,
) {
    for event in evr_mouse.read() {
        if let Ok((for_player, bp_id, mut ap)) = nf_nodes.get_mut(event.entity()) {
            get_assert!(for_player, players, |(quest_status,)| {
                match event.event_kind() {
                    MouseEventTtyKind::Moved => ap.set_timing(2.0),
                    MouseEventTtyKind::Exit => {
                        if get_assert!(bp_id, board_pieces)?
                            .map(|node_id| quest_status.is_node_done(node_id))
                            .unwrap_or(false)
                        {
                            ap.set_timing(1.0)
                        } else {
                            ap.set_timing(0.0);
                        }
                    },
                    MouseEventTtyKind::Down(input_event::MouseButton::Left) => {
                        log::debug!("Click event for NF node");
                    },
                    _ => {},
                }
                Some(())
            });
        }
    }
}
