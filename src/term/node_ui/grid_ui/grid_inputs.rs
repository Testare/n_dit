use std::ops::Deref;

use crossterm::event::{MouseButton, MouseEventKind};
use game_core::node::{
    ActiveCurio, Curio, CurrentTurn, InNode, Node, NodeOp, OnTeam, Pickup, Team, TeamPhase,
};
use game_core::player::{ForPlayer, Player};

use super::{GridUi, Scroll2D};
use crate::term::layout::LayoutEvent;
use crate::term::node_ui::{NodeCursor, SelectedAction, SelectedEntity};
use crate::term::prelude::*;

pub fn handle_layout_events(
    mut ev_mouse: EventReader<LayoutEvent>,
    ui: Query<(&ForPlayer, &Scroll2D), With<GridUi>>,
    mut players: Query<
        (
            &InNode,
            &OnTeam,
            &mut NodeCursor,
            &mut SelectedEntity,
            &mut SelectedAction,
        ),
        With<Player>,
    >,
    nodes: Query<(&EntityGrid, &ActiveCurio, &CurrentTurn), With<Node>>,
    teams: Query<&TeamPhase, With<Team>>,
    pickups: Query<(), With<Pickup>>,
    curios: Query<&OnTeam, With<Curio>>,
    mut ev_node_op: EventWriter<Op<NodeOp>>,
) {
    for event in ev_mouse.iter() {
        if let Ok((ForPlayer(player), scroll)) = ui.get(event.entity()) {
            if let MouseEventKind::Down(button) = event.event_kind() {
                log::debug!("Clicked on the grid");
                let (node, team, mut cursor, mut selected_entity, mut selected_action) = players
                    .get_mut(*player)
                    .expect("a player should have a node cursor if there is a grid ui");
                let (grid, active_curio, current_turn) = nodes
                    .get(**node)
                    .expect(" the player should be in a node with the required components");
                let team_phase = teams
                    .get(**team)
                    .expect("the player should be on a real team");
                let clicked_pos = event.pos() + **scroll;
                let clicked_node_pos = UVec2 {
                    x: clicked_pos.x / 3,
                    y: clicked_pos.y / 2,
                };

                if **cursor.deref() != clicked_node_pos {
                    **cursor = clicked_node_pos;
                }

                let is_controlling_active_curio =
                    active_curio.is_some() && **current_turn == **team;

                let now_selected_entity = grid.item_at(**cursor);
                if selected_entity.0 != now_selected_entity
                    && (now_selected_entity.is_some() || !is_controlling_active_curio)
                {
                    selected_entity.0 = now_selected_entity;
                    **selected_action = None;
                }

                if *button == MouseButton::Right {
                    log::debug!("That was a right click");
                    if is_controlling_active_curio {
                        let active_curio_id = active_curio.unwrap();
                        let head = grid.head(active_curio_id).unwrap();
                        // TODO A lot of this logic is duplicated in node_op. Finding a way
                        // to condense it would be great
                        if selected_action.is_some() {}
                        if head.manhattan_distance(cursor.deref()) == 1 {
                            // TODO Possible better UI: Clicking on any moveable-square will move the curio one space in that direction
                            let valid_move_target = grid.square_is_free(**cursor)
                                || if let Some(pt_key) = grid.item_at(**cursor) {
                                    pt_key == active_curio_id || pickups.contains(pt_key)
                                } else {
                                    false
                                };

                            if valid_move_target {
                                if let [Some(dir), _] = head.dirs_to(cursor.deref()) {
                                    ev_node_op
                                        .send(Op::new(*player, NodeOp::MoveActiveCurio { dir }));
                                    continue;
                                }
                            }
                        }
                        // TODO try defaulting to some action

                        // If there was an action selected,
                        // If it was adjacent to the curio and open, move it there
                        // else if there was a default action and this is a valid target,
                        // apply it
                    } else if *team_phase == TeamPhase::Play {
                        if let Some(pt_key) = grid.item_at(**cursor) {
                            if let Ok(curio_team) = curios.get(pt_key) {
                                if curio_team == team {
                                    ev_node_op.send(Op::new(
                                        *player,
                                        NodeOp::ActivateCurio { curio_id: pt_key },
                                    ));
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
