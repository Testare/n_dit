use super::{CurrentTurn, Node, NodeOp, OnTeam};
use crate::player::Player;
use crate::prelude::*;
use crate::NDitCoreSet;

pub struct NodeAiPlugin;

impl Plugin for NodeAiPlugin {
    fn build(&self, app: &mut App) {
        // Later might change this to be a post-commands op so that it sets up AI after player ends their turn
        app.add_systems(PreUpdate, sys_ai.in_set(NDitCoreSet::ProcessInputs));
    }
}

#[derive(Component)]
pub enum NodeBattleIntelligence {
    DoNothing,
    Lazy,
    Simple,
}

fn sys_ai(
    ai_players: IndexedQuery<OnTeam, (Entity, &NodeBattleIntelligence), With<Player>>,
    changed_turn_nodes: Query<AsDerefCopied<CurrentTurn>, (Changed<CurrentTurn>, With<Node>)>,
    mut evr_node_ops: EventWriter<Op<NodeOp>>,
) {
    for current_turn in changed_turn_nodes.iter() {
        if let Ok((id, intelligence)) = ai_players.get_for(current_turn) {
            match intelligence {
                NodeBattleIntelligence::DoNothing => {
                    NodeOp::EndTurn.for_p(id).send(&mut evr_node_ops);
                },
                NodeBattleIntelligence::Lazy => {
                    NodeOp::EndTurn.for_p(id).send(&mut evr_node_ops);
                },
                NodeBattleIntelligence::Simple => {
                    todo!("This level of intelligence has not yet been demonstrated by the game author")
                },
            }
        }
    }
}
