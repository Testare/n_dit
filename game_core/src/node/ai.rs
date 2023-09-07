use std::sync::mpsc::{Receiver, Sender, TryRecvError};
use std::sync::Mutex;
use std::thread::JoinHandle;
use std::time::Duration;

use bevy::ecs::query::WorldQuery;
use bevy::time::Time;

use super::{Curio, CurrentTurn, MovesTaken, NoOpAction, Node, NodeOp, NodePiece, OnTeam, Team};
use crate::card::{Action, ActionEffect, ActionRange, Actions, MovementSpeed};
use crate::player::Player;
use crate::prelude::*;
use crate::NDitCoreSet;

pub struct NodeAiPlugin;

impl Plugin for NodeAiPlugin {
    fn build(&self, app: &mut App) {
        // Later might change this to be a post-commands op so that it sets up AI after player ends their turn
        app.add_systems(PreUpdate, sys_ai_apply.in_set(NDitCoreSet::ProcessInputs))
            .add_systems(Update, sys_ai.in_set(NDitCoreSet::PostProcessCommands));
    }
}

#[derive(Component, Deref)]
pub struct SimpleAiCurioOrder(pub usize);

#[derive(Clone, Component, Copy, Debug)]
pub enum NodeBattleIntelligence {
    DoNothing,
    Lazy,
    Simple,
}

#[derive(Component, Debug, Default, DerefMut, Deref)]
pub struct AiThread {
    ai_thread: Option<AiThreadInternal>,
}

#[derive(Debug)]
pub struct AiThreadInternal {
    handle: JoinHandle<()>,
    events: Mutex<Receiver<(Op<NodeOp>, Duration)>>,
    pause_until: Duration,
}

fn sys_ai_apply(
    time: Res<Time>,
    mut ai_players: Query<(Entity, AsDerefMut<AiThread>)>,
    mut evr_node_ops: EventWriter<Op<NodeOp>>,
) {
    for (id, ai_internal) in ai_players.iter_mut() {
        let mut thread_finished = false;
        let ai_internal = ai_internal.into_inner();
        if let Some(AiThreadInternal {
            events,
            handle,
            pause_until,
        }) = ai_internal
        {
            let elapsed = time.elapsed();
            if elapsed < *pause_until {
                continue;
            }
            if let Ok(rx) = events.lock() {
                match rx.try_recv() {
                    Ok((op, pause)) => {
                        evr_node_ops.send(op);
                        *pause_until = elapsed + pause;
                    },
                    Err(TryRecvError::Empty) => {},
                    Err(TryRecvError::Disconnected) => {
                        if handle.is_finished() {
                            thread_finished = true;
                            NodeOp::EndTurn.for_p(id).send(&mut evr_node_ops);
                        }
                    },
                }
            }
        }
        if thread_finished {
            *ai_internal = None;
        }
    }
}

#[derive(WorldQuery)]
struct PieceQ {
    id: Entity,
    actions: AsDerefCloned<Actions>,
    movement: Option<&'static MovementSpeed>,
    ai_order: OrUsize<AsDerefCopied<SimpleAiCurioOrder>, 30>,
}

#[derive(WorldQuery)]
struct ActionQ {
    range: Copied<ActionRange>,
    effect: Copied<ActionEffect>,
}

fn sys_ai(
    no_op_action: Res<NoOpAction>,
    mut ai_players: IndexedQuery<
        OnTeam,
        (Entity, &NodeBattleIntelligence, AsDerefMut<AiThread>),
        With<Player>,
    >,
    changed_turn_nodes: Query<
        (AsDerefCopied<CurrentTurn>, &EntityGrid),
        (Changed<CurrentTurn>, With<Node>),
    >,
    actions: Query<(Entity, (Copied<ActionRange>, Copied<ActionEffect>)), With<Action>>,
    pieces: Query<(AsDerefCopied<OnTeam>, PieceQ), (With<NodePiece>, With<Curio>)>,
) {
    for (current_turn, grid) in changed_turn_nodes.iter() {
        if let Ok((id, intelligence, mut ai_thread)) = ai_players.get_for_mut(current_turn) {
            match intelligence {
                NodeBattleIntelligence::DoNothing => {
                    let (sx, rx) = std::sync::mpsc::channel();
                    *ai_thread = Some(AiThreadInternal {
                        handle: std::thread::spawn(|| {
                            std::thread::sleep(Duration::from_secs(3));
                            let _ = sx;
                        }),
                        events: Mutex::new(rx),
                        pause_until: default(),
                    });
                },
                NodeBattleIntelligence::Lazy => {
                    let (sx, rx) = std::sync::mpsc::channel();
                    let actions: HashMap<Entity, _> = actions.iter().collect();
                    let my_pieces: Vec<(Entity, Vec<Entity>)> = pieces
                        .iter()
                        .filter_map(|(team, piece)| {
                            (team == current_turn).then_some((piece.id, piece.actions))
                        })
                        .collect();
                    let enemy_pieces: Vec<(Entity, Vec<Entity>)> = pieces
                        .iter()
                        .filter_map(|(team, piece)| {
                            (team != current_turn).then_some((piece.id, piece.actions))
                        })
                        .collect();
                    let grid = grid.clone();

                    let no_op_action = no_op_action.0;
                    *ai_thread = Some(AiThreadInternal {
                        pause_until: default(),
                        events: Mutex::new(rx),
                        handle: std::thread::spawn(move || {
                            lazy_ai_script(
                                id,
                                no_op_action,
                                sx,
                                grid,
                                my_pieces,
                                actions,
                                enemy_pieces,
                            );
                        }),
                    });
                },
                NodeBattleIntelligence::Simple => {
                    let (sx, rx) = std::sync::mpsc::channel();
                    let actions: HashMap<Entity, _> = actions.iter().collect();
                    let my_pieces: Vec<(Entity, Vec<Entity>, Option<MovementSpeed>, usize)> =
                        pieces
                            .iter()
                            .filter_map(|(team, piece)| {
                                (team == current_turn).then_some((
                                    piece.id,
                                    piece.actions,
                                    piece.movement.cloned(),
                                    piece.ai_order,
                                ))
                            })
                            .collect();
                    let enemy_pieces: Vec<(Entity, Vec<Entity>)> = pieces
                        .iter()
                        .filter_map(|(team, piece)| {
                            (team != current_turn).then_some((piece.id, piece.actions))
                        })
                        .collect();
                    let grid = grid.clone();

                    let no_op_action = no_op_action.0;
                    *ai_thread = Some(AiThreadInternal {
                        pause_until: default(),
                        events: Mutex::new(rx),
                        handle: std::thread::spawn(move || {
                            simple_ai_script(
                                id,
                                no_op_action,
                                sx,
                                grid,
                                my_pieces,
                                actions,
                                enemy_pieces,
                            );
                        }),
                    });
                },
            }
        }
    }
}

// Other scripts for the future:
// Searches all possible movement spaces for the ability to attack
// Pathfinds towards nearest piece
// Search by all actions on all pieces, whichever does the most damage wins

// No pathfinding, simply moves in the direction of the nearest piece until it is within attack distance.
fn simple_ai_script(
    id: Entity,
    no_op_action: Entity,
    sx: Sender<(Op<NodeOp>, Duration)>,
    mut grid: EntityGrid,
    mut my_pieces: Vec<(Entity, Vec<Entity>, Option<MovementSpeed>, usize)>,
    actions: HashMap<Entity, (ActionRange, ActionEffect)>,
    enemy_pieces: Vec<(Entity, Vec<Entity>)>,
) {
    std::thread::sleep(Duration::from_millis(350));
    my_pieces.sort_by_key(|piece| piece.3);
    for piece in my_pieces {
        sx.send((
            NodeOp::ActivateCurio { curio_id: piece.0 }.for_p(id),
            Duration::from_millis(500),
        ));
        let mut grid_head = grid.head(piece.0).expect("piece should be in grid");
        if let Some(closest_enemy_pt) = enemy_pieces
            .iter()
            .flat_map(|(id, _)| grid.points(*id))
            .min_by_key(|pt| pt.manhattan_distance(&grid_head))
        {
            log::trace!(
                "Closest enemy point for piece[{:?}] is {:?}",
                piece.1,
                closest_enemy_pt
            );
            let movement_speed = piece.2.as_ref().map(|ms| **ms).unwrap_or(0);
            for _ in 0..movement_speed {
                match grid_head.dirs_to(&closest_enemy_pt) {
                    [Some(dir1), Some(dir2)] => {
                        // Choose which to prioritize, then try one, and if it fails, the other.
                        let (dir1, dir2) = if grid_head
                            .dist_to_pt_along_compass(&closest_enemy_pt, dir1)
                            > grid_head.dist_to_pt_along_compass(&closest_enemy_pt, dir2)
                        {
                            (dir1, dir2)
                        } else {
                            (dir2, dir1)
                        };
                        if grid.square_is_free(grid_head + dir1)
                            || grid.item_at(grid_head + dir1) == Some(id)
                        {
                            log::trace!("{:?} Went {:?} from {:?}", piece.0, dir1, grid_head);
                            grid_head = grid_head + dir1;
                            sx.send((
                                NodeOp::MoveActiveCurio { dir: dir1 }.for_p(id),
                                Duration::from_millis(400),
                            ));
                        } else if grid.square_is_free(grid_head + dir2)
                            || grid.item_at(grid_head + dir2) == Some(id)
                        {
                            log::trace!("{:?} Went {:?} from {:?}", piece.0, dir2, grid_head);
                            grid_head = grid_head + dir2;
                            sx.send((
                                NodeOp::MoveActiveCurio { dir: dir2 }.for_p(id),
                                Duration::from_millis(400),
                            ));
                        } else {
                            log::trace!(
                                "{:?} can't go {:?} OR {:?} from {:?}",
                                piece.0,
                                dir1,
                                dir2,
                                grid_head
                            );
                            break;
                        }
                    },
                    [Some(dir), None] => {
                        // if dir is not blocked ,go that way
                        if grid.square_is_free(grid_head + dir)
                            || grid.item_at(grid_head + dir) == Some(id)
                        {
                            log::trace!("{:?} Went {:?} from {:?}", piece.0, dir, grid_head);
                            grid_head = grid_head + dir;
                            sx.send((
                                NodeOp::MoveActiveCurio { dir }.for_p(id),
                                Duration::from_millis(400),
                            ));
                        } else {
                            log::trace!("{:?} can't go {:?} from {:?}", piece.0, dir, grid_head);
                            break;
                        }
                    },
                    [None, None] => {
                        panic!("Somehow the grid points became super imposed on each other")
                    },
                    [None, Some(_)] => unreachable!(),
                }
            }
        }

        if let Some((action, target, target_id, dmg)) = piece.1.into_iter().find_map(|action| {
            let (range, effect) = actions.get(&action)?;
            match effect {
                ActionEffect::Damage(dmg) => {
                    for enemy_piece in enemy_pieces.iter() {
                        let enemy_squares = grid.points(enemy_piece.0);
                        if let Some(point_in_range) = range.pt_in_range(enemy_squares, grid_head) {
                            return Some((action, point_in_range, enemy_piece.0, *dmg));
                        }
                    }
                    None
                },
                ActionEffect::Heal(_) => None,
            }
        }) {
            sx.send((
                NodeOp::PerformCurioAction {
                    action,
                    curio: None,
                    target,
                }
                .for_p(id),
                Duration::from_secs(2),
            ));
            grid.pop_back_n(target_id, dmg);
        } else {
            sx.send((
                NodeOp::PerformCurioAction {
                    action: no_op_action,
                    curio: None,
                    target: UVec2::default(),
                }
                .for_p(id),
                Duration::from_millis(500),
            ));
        }
    }
}

// Attacks whatever is nearby, no movement
fn lazy_ai_script(
    id: Entity,
    no_op_action: Entity,
    sx: Sender<(Op<NodeOp>, Duration)>,
    grid: EntityGrid,
    my_pieces: Vec<(Entity, Vec<Entity>)>,
    actions: HashMap<Entity, (ActionRange, ActionEffect)>,
    enemy_pieces: Vec<(Entity, Vec<Entity>)>,
) {
    std::thread::sleep(Duration::from_millis(350));
    for piece in my_pieces {
        sx.send((
            NodeOp::ActivateCurio { curio_id: piece.0 }.for_p(id),
            Duration::from_millis(500),
        ));
        if let Some((action, target)) = piece.1.into_iter().find_map(|action| {
            let (range, effect) = actions.get(&action)?;
            match effect {
                ActionEffect::Damage(_) => {
                    let head = grid.head(piece.0).expect("ai piece should have a head");
                    for enemy_piece in enemy_pieces.iter() {
                        let enemy_squares = grid.points(enemy_piece.0);
                        if let Some(point_in_range) = range.pt_in_range(enemy_squares, head) {
                            return Some((action, point_in_range));
                        }
                    }
                    None
                },
                ActionEffect::Heal(_) => None,
            }
        }) {
            sx.send((
                NodeOp::PerformCurioAction {
                    action,
                    curio: None,
                    target,
                }
                .for_p(id),
                Duration::from_secs(2),
            ));
        } else {
            sx.send((
                NodeOp::PerformCurioAction {
                    action: no_op_action,
                    curio: None,
                    target: UVec2::default(),
                }
                .for_p(id),
                Duration::from_millis(500),
            ));
        }
    }
}
