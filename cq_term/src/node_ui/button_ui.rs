use bevy::ecs::query::Has;
use crossterm::event::{MouseButton, MouseEventKind};
use game_core::node::{
    AccessPoint, AccessPointLoadingRule, IsReadyToGo, Node, NodeOp, NodePiece, OnTeam, Teams,
};
use game_core::op::OpResult;
use game_core::player::{ForPlayer, Player};

use crate::layout::{LayoutEvent, LayoutMouseTargetDisabled};
use crate::prelude::*;

#[derive(Clone, Copy, Component, Reflect)]
pub struct ReadyButton;

#[derive(Clone, Copy, Component, Reflect)]
pub struct EndTurnButton;

pub fn mouse_ready_button(
    mut evr_mouse: EventReader<LayoutEvent>,
    ready_button: Query<&ForPlayer, With<ReadyButton>>,
    mut evw_node_op: EventWriter<Op<NodeOp>>,
) {
    for mouse_event in evr_mouse.iter() {
        if !matches!(
            mouse_event.event_kind(),
            MouseEventKind::Down(MouseButton::Left)
        ) {
            continue;
        }
        if let Ok(for_player) = ready_button.get(mouse_event.entity()) {
            NodeOp::ReadyToGo.for_p(**for_player).send(&mut evw_node_op);
        }
    }
}

pub fn sys_ready_button_disable(
    mut commands: Commands,
    mut ev_node_op_result: EventReader<OpResult<NodeOp>>,
    ready_buttons: IndexedQuery<
        ForPlayer,
        (Entity, Has<LayoutMouseTargetDisabled>),
        With<ReadyButton>,
    >,
    nodes: Query<(&AccessPointLoadingRule, &EntityGrid, &Teams), With<Node>>,
    players: Query<(Option<&IsReadyToGo>,), With<Player>>,
    access_points: Query<(&ForPlayer, Entity, &OnTeam, &AccessPoint), With<NodePiece>>,
) {
    // TODO make this a run condition?
    // for OpResult { .. }
    for node_op_result in ev_node_op_result.iter() {
        if let OpResult {
            result: Ok(_),
            source: Op { player, op },
        } = node_op_result
        {
            let should_be_enabled = match op {
                NodeOp::LoadAccessPoint { .. } => true,
                NodeOp::ReadyToGo => false,
                NodeOp::UnloadAccessPoint { .. } => {
                    // TODO check if other accesss points are still loaded
                    // TODO actually emit this nodeopresult
                    false
                },
                _ => continue,
            };
            if let Ok((id, button_is_disabled)) = ready_buttons.get_for(*player) {
                if button_is_disabled && should_be_enabled {
                    commands.entity(id).remove::<LayoutMouseTargetDisabled>();
                } else if !button_is_disabled && !should_be_enabled {
                    commands.entity(id).insert(LayoutMouseTargetDisabled);
                }
            }
        }
    }
}
