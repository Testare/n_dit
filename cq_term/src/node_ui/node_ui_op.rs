use game_core::node::{InNode, Node};
use game_core::op::OpSubtype;
use game_core::player::Player;

use super::{NodeCursor, SelectedAction, SelectedEntity};
use crate::prelude::*;

#[derive(Clone, Debug)]
pub enum NodeUiOp {
    ChangeFocus(FocusTarget),
    MoveNodeCursor(CompassOrPoint),
    MoveMenuCursor,
    SetSelectedAction,
}

#[derive(Clone, Copy, Debug)]
pub enum FocusTarget {
    Next,
    Prev,
    Grid,
    CardMenu,
    ActionMenu,
}

impl OpSubtype for NodeUiOp {
    type Error = ();
}

pub fn sys_process_ui_op_move_cursor(
    mut ev_node_ui_op: EventReader<Op<NodeUiOp>>,
    mut players: Query<
        (
            &InNode,
            &mut NodeCursor,
            &mut SelectedEntity,
            &mut SelectedAction,
        ),
        With<Player>,
    >,
    nodes: Query<(&EntityGrid,), With<Node>>,
) {
    for Op { player, op } in ev_node_ui_op.iter() {
        if let NodeUiOp::MoveNodeCursor(compass_or_point) = op {
            get_assert_mut!(*player, &mut players, |(
                InNode(node),
                mut cursor,
                selected_entity,
                selected_action,
            )| {
                let (grid,) = get_assert!(*node, nodes)?;

                let next_pt = compass_or_point.point_from(**cursor);
                if **cursor != next_pt {
                    **cursor = next_pt;
                }
                cursor.adjust_to_self(selected_entity, selected_action, grid);
                Some(())
            });
        }
    }
}
