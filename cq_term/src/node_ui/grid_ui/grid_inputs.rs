use bevy::ecs::system::RunSystemOnce;
use game_core::card::{Action, Actions, NO_OP_ACTION_ID};
use game_core::node::{
    ActiveCurio, Curio, CurrentTurn, InNode, IsTapped, Node, NodeOp, NodePiece, OnTeam, Team,
    TeamPhase,
};
use game_core::op::CoreOps;
use game_core::player::{ForPlayer, Player};

use super::{calculate_ui_components, GridHoverPoint, GridUi, LastGridHoverPoint, Scroll2d};
use crate::base_ui::context_menu::{ContextAction, ContextActions};
use crate::input_event::{MouseEventTty, MouseEventTtyKind};
use crate::key_map::NamedInput;
use crate::layout::UiFocus;
use crate::node_ui::node_ui_op::{FocusTarget, UiOps};
use crate::node_ui::{
    AvailableActionTargets, AvailableMoves, NodeCursor, NodeUiOp, SelectedAction, SelectedEntity,
};
use crate::prelude::*;
use crate::{KeyMap, Submap};

#[derive(Resource)]
pub struct GridContextActions {
    move_here: Entity,
    perform_action: Entity,
    select_piece: Entity,
    select_square: Entity,
}

impl FromWorld for GridContextActions {
    fn from_world(world: &mut World) -> Self {
        let move_here = world.spawn((
            Name::new("Move here CA"),
            ContextAction::new_with_mouse_event("Move here", |grid_id, src_mouse_event, world| {
                world.run_system_once(move |
                    mut res_core_ops: ResMut<CoreOps>,
                    q_grid_ui: Query<(AsDerefCopied<ForPlayer>, AsDerefCopied<Scroll2d>), With<GridUi>>,
                    q_player: Query<(&InNode, &SelectedEntity, &AvailableMoves), With<Player>>,
                    q_node: Query<&ActiveCurio, With<Node>>,
                    | {
                        (||{ // try
                            let (player_id, scroll) = get_assert!(grid_id, q_grid_ui)?;
                            let (&InNode(node_id), &SelectedEntity(selected_piece), available_moves,) = get_assert!(player_id, q_player)?;
                            let &ActiveCurio(active_curio) = get_assert!(node_id, q_node)?;

                            let mut path = Vec::new();
                            let mut pt = calculate_ui_components::calculate_grid_pt(src_mouse_event.relative_pos(), scroll);
                            while let Some(&Some(dir)) = available_moves.get(&pt) {
                                path.push(dir);
                                pt = pt - dir;
                            }
                            if path.is_empty() {
                                return None;
                            }
                            if selected_piece != active_curio {
                                if let Some(selected_piece) = selected_piece {
                                    let curio_id = selected_piece;
                                    res_core_ops.request(player_id, NodeOp::ActivateCurio { curio_id })
                                }
                            }
                            // TODO Possibly configurable idea: Clicking moves you one space in that direction each time, instead of every step.
                            for &dir in path.iter().rev() {
                                res_core_ops.request(player_id, NodeOp::MoveActiveCurio { dir })
                            }

                            Some(())
                        })();
                    }
                );
            })
        )).id();
        let perform_action = world
            .spawn((
                Name::new("Perform action CA"),
                ContextAction::new("Perform action", |grid_id, world| {
                    // Once we can run systems with input we make this a bit easier
                    world.run_system_once(
                        move |ast_action: Res<Assets<Action>>,
                              mut res_core_ops: ResMut<CoreOps>,
                              q_grid_ui: Query<(&ForPlayer, &LastGridHoverPoint), With<GridUi>>,
                              q_player: Query<(&SelectedAction, &SelectedEntity), With<Player>>,
                              q_curio: Query<&Actions, With<Curio>>| {
                            get_assert!(grid_id, q_grid_ui).and_then(
                                |(&ForPlayer(player_id), &LastGridHoverPoint(target))| {
                                    let (&SelectedAction(action_index), &SelectedEntity(curio)) =
                                        get_assert!(player_id, q_player)?;
                                    let actions = q_curio.get(curio?).ok()?;
                                    let action_handle = actions.get(action_index?)?;
                                    let action_id = ast_action.get(action_handle)?.id_cow();

                                    let op = NodeOp::PerformCurioAction {
                                        action_id,
                                        curio,
                                        target,
                                    };
                                    res_core_ops.request(player_id, op);
                                    Some(())
                                },
                            );
                        },
                    )
                }),
            ))
            .id();
        let select_piece = world
            .spawn((
                Name::new("Select piece CA"),
                ContextAction::new("Select piece", |grid_id, world| {
                    // Note: Identical to "Select square CA"
                    (|| {
                        // trying "try" block
                        let &ForPlayer(player_id) = world.get(grid_id)?;
                        let &LastGridHoverPoint(target) = world.get(grid_id)?;
                        let op = NodeUiOp::MoveNodeCursor(CompassOrPoint::Point(target));
                        world.resource_mut::<CoreOps>().request(player_id, op);
                        Some(())
                    })();
                }),
            ))
            .id();
        let select_square = world
            .spawn((
                Name::new("Select square CA"),
                ContextAction::new("Select square", |grid_id, world| {
                    // Note: Identical to "Select piece CA"
                    (|| {
                        // trying "try" block
                        let &ForPlayer(player_id) = world.get(grid_id)?;
                        let &LastGridHoverPoint(target) = world.get(grid_id)?;
                        let op = NodeUiOp::MoveNodeCursor(CompassOrPoint::Point(target));
                        world.resource_mut::<CoreOps>().request(player_id, op);
                        Some(())
                    })();
                }),
            ))
            .id();
        Self {
            move_here,
            perform_action,
            select_piece,
            select_square,
        }
    }
}

pub fn sys_grid_context_actions(
    res_grid_context_actions: Res<GridContextActions>,
    q_player: Query<
        (
            &AvailableMoves,
            &AvailableActionTargets,
            &SelectedEntity,
            &InNode,
            &OnTeam,
            &NodeCursor,
        ),
        With<Player>,
    >,
    q_node: Query<(&EntityGrid, &CurrentTurn, &ActiveCurio), With<Node>>,
    q_team: Query<&TeamPhase, With<Team>>,
    q_curio: Query<AsDerefCopied<OnTeam>, With<Curio>>,
    mut q_grid_ui: Query<
        (
            &ForPlayer,
            AsDerefCopied<GridHoverPoint>,
            &mut ContextActions,
        ),
        With<GridUi>,
    >,
) {
    for (&ForPlayer(player_id), grid_hover_point, mut context_actions) in q_grid_ui.iter_mut() {
        let actions = grid_hover_point
            .and_then(|grid_hover_point| {
                let mut actions = Vec::new();
                let (
                    available_moves,
                    available_action_targets,
                    &SelectedEntity(selected_piece),
                    &InNode(node_id),
                    &OnTeam(team_id),
                    &NodeCursor(node_cursor),
                ) = get_assert!(player_id, q_player)?;
                let (grid, &CurrentTurn(current_turn), &ActiveCurio(active_curio)) =
                    get_assert!(node_id, q_node)?;
                let moving_piece = selected_piece.or(active_curio); // The piece we're showing available moves for
                let cursor_at_head =
                    moving_piece.and_then(|piece| grid.head(piece)) == Some(grid_hover_point);
                let moving_piece_team_id = moving_piece.and_then(|piece| q_curio.get(piece).ok());
                let &team_phase = get_assert!(team_id, q_team)?;
                // TODO do not show these actions for non-team selected pieces
                if current_turn == team_id
                    && moving_piece_team_id == Some(team_id)
                    && team_phase == TeamPhase::Play
                {
                    if available_action_targets
                        .get(&grid_hover_point)
                        .copied()
                        .unwrap_or_default()
                    {
                        actions.push(res_grid_context_actions.perform_action);
                    }
                    if available_moves.contains_key(&grid_hover_point) && !cursor_at_head {
                        actions.push(res_grid_context_actions.move_here);
                    }
                }
                if node_cursor != grid_hover_point {
                    if grid.item_at(grid_hover_point).is_some() {
                        actions.push(res_grid_context_actions.select_piece);
                    } else {
                        actions.push(res_grid_context_actions.select_square);
                    }
                }
                Some(actions)
            })
            .unwrap_or_default();
        *context_actions.actions_mut() = actions;
    }
    // I need to make sure that the grid remembers where the grid hover point was when we perform the context actione if we display the context action menu

    // If points is in available_moves - "Move here"
    // Else if point is in attack range - "Apply action"
    //
    // Later we can try updating the string (I can't just call it "Attack piece" since not all actions are attacks,
    // and not all targets are pieces). But to do that we should update ContextActions to be able to generate a name
    // based on the input
    //
    //
    //

    // If HoverGridPoint is over NodeCurosr
    // - Nothing
    // + Else If hoverGridPoint is over a square containing an entity,
    // "Select piece" CA (Even if it is ours)
    // - Else "Move cursor to point"
}

pub fn handle_layout_events(
    mut res_ui_ops: ResMut<UiOps>,
    mut evr_mouse: EventReader<MouseEventTty>,
    q_grid_ui: Query<AsDerefCopied<ForPlayer>, With<GridUi>>,
) {
    for event in evr_mouse.read() {
        if let Ok(player_id) = q_grid_ui.get(event.entity()) {
            if let (MouseEventTtyKind::Down(_), true) = (event.event_kind(), event.is_top_entity())
            {
                res_ui_ops.request(player_id, NodeUiOp::ChangeFocus(FocusTarget::Grid));
            }
        }
    }
}

pub fn kb_grid(
    ast_actions: Res<Assets<Action>>,
    mut res_core_ops: ResMut<CoreOps>,
    mut res_ui_ops: ResMut<UiOps>,
    mut ev_keys: EventReader<KeyEvent>,
    nodes: Query<(&EntityGrid, &ActiveCurio, &CurrentTurn), With<Node>>,
    players: Query<
        (
            Entity, // Solid candidate for making a WorldQuery derive
            &InNode,
            &OnTeam,
            &UiFocus,
            &KeyMap,
            &NodeCursor,
            &SelectedEntity,
            &SelectedAction,
        ),
        With<Player>,
    >,
    node_pieces: Query<(Option<&Actions>, &IsTapped), With<NodePiece>>,
    grid_uis: Query<(), With<GridUi>>,
    teams: Query<&TeamPhase, With<Team>>,
) {
    for KeyEvent { code, modifiers } in ev_keys.read() {
        for (
            player,
            InNode(node),
            OnTeam(team),
            UiFocus(focus_opt),
            key_map,
            cursor,
            selected_entity,
            selected_action,
        ) in players.iter()
        {
            if focus_opt
                .map(|focused_ui| !grid_uis.contains(focused_ui))
                .unwrap_or(false)
            {
                // If there is a focus and it isn't grid_ui, don't do grid_ui controls
                continue;
            }

            key_map
                .named_input_for_key(Submap::Node, *code, *modifiers)
                .and_then(|named_input| {
                    let (grid, active_curio, turn) = get_assert!(*node, nodes)?;
                    let is_controlling_active_curio = active_curio.is_some() && **turn == *team;
                    let team_phase = teams.get(*team).ok()?;

                    match named_input {
                        NamedInput::Direction(dir) => {
                            if is_controlling_active_curio && selected_action.is_none() {
                                res_core_ops.request(player, NodeOp::MoveActiveCurio { dir })
                            } else {
                                res_ui_ops.request(player, NodeUiOp::MoveNodeCursor(dir.into()));
                            }
                        },
                        NamedInput::Activate => {
                            if let Some(selected_action_index) = **selected_action {
                                selected_entity.of(&node_pieces).and_then(
                                    |(actions, is_tapped)| {
                                        if **is_tapped || *team_phase == TeamPhase::Setup {
                                            return None;
                                        }
                                        let action = ast_actions.get(actions?.get(selected_action_index)?)?;
                                        res_core_ops.request(
                                            player,
                                            NodeOp::PerformCurioAction {
                                                action_id: action.id_cow(),
                                                curio: **selected_entity,
                                                target: **cursor,
                                            },
                                        );
                                        Some(())
                                    },
                                );
                            } else if is_controlling_active_curio {
                                selected_entity.of(&node_pieces).and_then(
                                    |(actions, is_tapped)| {
                                        if **is_tapped {
                                            return None;
                                        }
                                        match actions.map(|actions| (actions.len(), actions)) {
                                            None | Some((0, _)) => {
                                                res_core_ops.request(
                                                    player,
                                                    NodeOp::PerformCurioAction {
                                                        action_id: NO_OP_ACTION_ID,
                                                        curio: **selected_entity,
                                                        target: default(),
                                                    },
                                                );
                                            },
                                            Some((1, actions)) => {
                                                if let Some(action) = ast_actions.get(actions.0.first().expect("if the len is 1, there should be an action at 0")) {
                                                    if action.range().is_none() {
                                                        res_core_ops.request(
                                                            player,
                                                            NodeOp::PerformCurioAction {
                                                                action_id: action.id_cow(),
                                                                curio: **selected_entity,
                                                                target: default(),
                                                            },
                                                        );
                                                    }
                                                }
                                            },
                                            _ => {},
                                        }
                                        Some(())
                                    },
                                );
                            // If the curio has an action menu, focus on it
                            } else if let Some(curio_id) = **selected_entity {
                                if **turn == *team && *team_phase != TeamPhase::Setup {
                                    res_core_ops.request(player, NodeOp::ActivateCurio { curio_id });
                                }
                            }
                        },
                        NamedInput::Undo => {
                            if selected_action.is_some() {
                                res_ui_ops.request(player, NodeUiOp::SetSelectedAction(None));
                                if is_controlling_active_curio {
                                    active_curio.and_then(|active_curio_id| {
                                        let head = grid.head(active_curio_id)?;
                                        res_ui_ops.request(player, NodeUiOp::MoveNodeCursor(head.into()));
                                        Some(())
                                    });
                                }
                            }
                        },
                        _ => {},
                    }
                    Some(())
                }
            );
        }
    }
}
