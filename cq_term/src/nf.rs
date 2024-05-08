use bevy_yarnspinner::events::ExecuteCommandEvent;
use bevy_yarnspinner::prelude::DialogueRunner;
use game_core::board::BoardPiece;
use game_core::dialog::Dialog;
use game_core::node::{self, ForNode, NodeId, NodeOp, VictoryStatus};
use game_core::op::{CoreOps, OpResult};
use game_core::player::{ForPlayer, Ncp, Player};
use game_core::quest::QuestStatus;
use game_core::shop::InShop;

use crate::animation::AnimationPlayer;
use crate::base_ui::context_menu::{ContextAction, ContextActions};
use crate::base_ui::HoverPoint;
use crate::board_ui::{ActionsPanelIgnoredAction, BoardPieceUi, SelectedBoardPieceUi};
use crate::input_event::MouseEventListener;
use crate::layout::VisibilityTty;
use crate::prelude::*;

#[derive(Debug)]
pub struct NfPlugin;

pub const NODE_ANIMATION_FRAME_UNBEATEN: f32 = 0.0;
pub const NODE_ANIMATION_FRAME_COMPLETE: f32 = 1.0;
pub const NODE_ANIMATION_FRAME_HOVER: f32 = 2.0;
pub const NODE_ANIMATION_FRAME_SELECTED: f32 = 3.0;

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
                    sys_nf_node_ui_display,
                    sys_nf_victory_dialog,
                    sys_yarn_commands_nf,
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
                ContextAction::new("Enter Shop", |id, world| {
                    (|| {
                        let &ForPlayer(player_id) = world.get(id)?;

                        // Do not allow enter shop when already in shop
                        if world.get::<InShop>(player_id).is_some() {
                            return None;
                        }

                        let &BoardPieceUi(bp_id) = world.get(id)?;
                        let nf_shop: &NFShop = world.get(bp_id)?;
                        let dialog_id = nf_shop.dialog_id();
                        let mut player_dr = world.get_mut::<DialogueRunner>(player_id)?;
                        if !player_dr.is_running() {
                            player_dr.start_node(dialog_id);
                        }
                        Some(())
                    })();
                }),
            ))
            .id();
        let select_piece = world
            .spawn((
                Name::new("Select Node CA"),
                ActionsPanelIgnoredAction,
                ContextAction::new("Select Node", |id, world| {
                    (|| {
                        // try
                        let &ForPlayer(player_id) = world.get(id)?;
                        // Do not allow selecting pieces while in dialog
                        if let Some(dialog) = world.get::<Dialog>(player_id) {
                            if dialog.line().is_some() {
                                return None;
                            }
                        }

                        // Do not allow selecting pieces while in shop
                        if world.get::<InShop>(player_id).is_some() {
                            return None;
                        }

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

#[derive(Component, Debug, Deref)]
pub struct NFShop(pub String);

impl NFShop {
    fn dialog_id(&self) -> String {
        self.0.replace(':', "_")
    }
}

#[derive(Component, Debug)]
struct NFNodeUi;

#[derive(Component, Debug)]
pub struct VictoryDialogue(pub String);

impl VictoryDialogue {
    pub fn new(dialog_id: &str) -> Self {
        Self(dialog_id.to_string())
    }
}

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
                    HoverPoint::default(),
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
                    ap.set_timing(NODE_ANIMATION_FRAME_COMPLETE);
                } else {
                    ap.set_timing(NODE_ANIMATION_FRAME_UNBEATEN);
                }
                Some(())
            });
        }
    }
}

fn sys_nf_node_ui_display(
    q_player: Query<
        (
            Entity,
            Ref<QuestStatus>,
            AsDerefCopied<SelectedBoardPieceUi>,
        ),
        (With<Player>, With<Ncp>),
    >,
    nf_nodes: Query<(Option<AsDeref<RequiredNodes>>, Option<AsDeref<ForNode>>), With<NFNode>>,
    mut nf_node_ui: Query<
        (
            Entity,
            AsDerefCopied<ForPlayer>,
            AsDerefCopied<BoardPieceUi>,
            &mut AnimationPlayer,
            AsDerefCopied<HoverPoint>,
            AsDerefMut<VisibilityTty>,
        ),
        With<NFNodeUi>,
    >,
) {
    for (player_id, quest_status, selected_board_piece) in q_player.iter() {
        for (bp_ui_id, for_player, bp_id, mut ap, hover_point, mut is_visible) in
            nf_node_ui.iter_mut()
        {
            if player_id != for_player {
                continue;
            } // TODO try inverting these
            let next_timing = get_assert!(bp_id, nf_nodes, |(required_nodes, for_node)| {
                if !*is_visible {
                    // It is assumed that we only go from invisible to visible
                    let met_requirements = required_nodes
                        .map(|req_nodes| {
                            req_nodes
                                .iter()
                                .all(|node_id| quest_status.is_node_done(node_id))
                        })
                        .unwrap_or(true);
                    if met_requirements {
                        *is_visible = true;
                    } else {
                        return None;
                    }
                }

                if selected_board_piece == Some(bp_ui_id) {
                    Some(NODE_ANIMATION_FRAME_SELECTED)
                } else if hover_point.is_some() {
                    Some(NODE_ANIMATION_FRAME_HOVER)
                } else if for_node
                    .map(|for_node| quest_status.is_node_done(for_node))
                    .unwrap_or(false)
                {
                    Some(NODE_ANIMATION_FRAME_COMPLETE)
                } else {
                    Some(NODE_ANIMATION_FRAME_UNBEATEN)
                }
            });

            if let Some(next_timing) = next_timing {
                if ap.timing() != next_timing {
                    ap.set_timing(next_timing);
                }
            }
        }
    }
}

pub fn sys_nf_victory_dialog(
    mut evr_node_op: EventReader<OpResult<NodeOp>>,
    q_nf_node: Query<(AsDeref<ForNode>, &VictoryDialogue), With<NFNode>>,
    mut q_player: Query<&mut DialogueRunner>,
) {
    for node_op_result in evr_node_op.read() {
        if let OpResult {
            op: NodeOp::QuitNode(node_sid),
            source: player_id,
            result: Ok(metadata),
        } = node_op_result
        {
            (|| {
                let victory_status = metadata.get_required(node::key::VICTORY_STATUS).ok()?;
                if matches!(
                    victory_status,
                    VictoryStatus::Loss | VictoryStatus::Undecided
                ) {
                    return None;
                }
                let first_victory = metadata.get_required(node::key::FIRST_VICTORY).ok()?;
                if first_victory {
                    let (_, victory_dialog) = q_nf_node.iter().find(|i| i.0 == node_sid)?;
                    let mut player_dr = q_player.get_mut(*player_id).ok()?;
                    player_dr.start_node(victory_dialog.0.as_str());
                }
                Some(())
            })();
        }
    }
}

fn sys_yarn_commands_nf(
    mut evr_yarn_commands: EventReader<ExecuteCommandEvent>,
    mut q_player: Query<AsDerefMut<SelectedBoardPieceUi>, With<Player>>,
    q_nf_node: Query<(AsDeref<ForNode>, Entity), With<NFNode>>,
    q_nf_shop: Query<(AsDeref<NFShop>, Entity), With<NFShop>>,
    q_board_piece: Query<(AsDerefCopied<BoardPieceUi>, Entity)>,
) {
    for ExecuteCommandEvent { command, source } in evr_yarn_commands.read() {
        if command.name.as_str() != "reveal_node" {
            continue;
        }
        let result: Result<(), String> = (|| {
            // try
            let sid_str = command
                .parameters
                .first()
                .ok_or("Missing node parameter".to_string())?
                .to_string();

            let node_bp_id = if sid_str.starts_with("node:") {
                let node_sid: SetId = sid_str
                    .parse()
                    .map_err(|e| format!("Error parsing {sid_str:?}: {e:?}"))?;
                let node_sid: NodeId = NodeId::from(node_sid);
                let (_, node_bp_id) = q_nf_node
                    .iter()
                    .find(|i| i.0 == &node_sid)
                    .ok_or_else(|| format!("Cannot find node piece that matches {node_sid:?}"))?;
                node_bp_id
            } else if sid_str.starts_with("warez:") {
                let (_, node_bp_id) = q_nf_shop
                    .iter()
                    .find(|i| i.0 == &sid_str)
                    .ok_or_else(|| format!("Cannot find shop piece that matches {sid_str:?}"))?;
                node_bp_id
            } else {
                return Err(format!("Don't know what type of piece {sid_str} is"));
            };

            let (_, node_bpui_id) = q_board_piece
                .iter()
                .find(|i| i.0 == node_bp_id)
                .ok_or_else(|| {
                    format!("Cannot find piece UI corresponding to piece {node_bp_id:?}")
                })?;
            let mut player_selected_bp = q_player
                .get_mut(*source)
                .map_err(|_| format!("Unable to find UI for player {source:?}"))?;
            *player_selected_bp = Some(node_bpui_id);
            Ok(())
        })();

        if let Err(msg) = result {
            log::error!("Error with yarn reveal_node command: {msg}");
        }
    }
}
