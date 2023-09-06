use std::f32::consts::E;
use std::sync::mpsc::{Receiver, TryRecvError};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::Duration;

use bevy::time::Time;
use bevy::utils::Instant;

use super::{CurrentTurn, NoOpAction, Node, NodeOp, NodePiece, OnTeam};
use crate::card::{Action, ActionEffect, ActionRange, Actions};
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
    mut time: Res<Time>,
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
    // mut evr_node_ops: EventWriter<Op<NodeOp>>,
    actions: Query<(Entity, (Copied<ActionRange>, Copied<ActionEffect>)), With<Action>>,
    pieces: Query<(AsDerefCopied<OnTeam>, (Entity, AsDerefCloned<Actions>)), With<NodePiece>>,
) {
    for (current_turn, grid) in changed_turn_nodes.iter() {
        if let Ok((id, intelligence, mut ai_thread)) = ai_players.get_for_mut(current_turn) {
            match intelligence {
                NodeBattleIntelligence::DoNothing => {
                    // NodeOp::EndTurn.for_p(id).send(&mut evr_node_ops);
                    let (sx, rx) = std::sync::mpsc::channel();
                    *ai_thread = Some(AiThreadInternal {
                        handle: std::thread::spawn(|| {
                            let _ = sx;
                            std::thread::sleep(Duration::from_secs(3));
                            // NodeOp::EndTurn.for_p(id).send(&mut evr_node_ops);
                            // sx.
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
                        .filter_map(|(team, piece)| (team == current_turn).then_some(piece))
                        .collect();
                    let enemy_pieces: Vec<(Entity, Vec<Entity>)> = pieces
                        .iter()
                        .filter_map(|(team, piece)| (team != current_turn).then_some(piece))
                        .collect();
                    let grid = grid.clone();

                    let no_op_action = no_op_action.0;
                    *ai_thread = Some(AiThreadInternal {
                        pause_until: default(),
                        events: Mutex::new(rx),
                        handle: std::thread::spawn(move || {
                            if let Some(action) = actions.iter().next() {}
                            std::thread::sleep(Duration::from_millis(350));
                            for piece in my_pieces {
                                sx.send((
                                    NodeOp::ActivateCurio { curio_id: piece.0 }.for_p(id),
                                    Duration::from_millis(500),
                                ));
                                if let Some((action, target)) =
                                    piece.1.into_iter().find_map(|action| {
                                        let (range, effect) = actions.get(&action)?;
                                        match effect {
                                            ActionEffect::Damage(_) => {
                                                let head = grid
                                                    .head(piece.0)
                                                    .expect("ai piece should have a head");
                                                for enemy_piece in enemy_pieces.iter() {
                                                    let enemy_squares = grid.points(enemy_piece.0);
                                                    if let Some(point_in_range) =
                                                        range.pt_in_range(enemy_squares, head)
                                                    {
                                                        return Some((action, point_in_range));
                                                    }
                                                }
                                                None
                                            },
                                            ActionEffect::Heal(_) => None,
                                        }
                                    })
                                {
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
                            let _ = sx;
                        }),
                    });
                },
                NodeBattleIntelligence::Simple => {
                    todo!("Testare is not this smart yet")
                },
            }
        }
    }
}
