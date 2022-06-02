use super::{Node, NodeChange, Team};
use serde::{Deserialize, Serialize};

mod pathfinding;
mod simple_ai;

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub enum EnemyAi {
    Simple,
}

impl EnemyAi {
    pub fn generate_enemy_ai_actions<C: FnMut(NodeChange)>(&self, node: Node, collect: C) {
        let keys = node.curio_keys_for_team(Team::EnemyTeam);
        match node.enemy_ai() {
            EnemyAi::Simple => simple_ai::generate_enemy_ai_actions(node, keys, collect),
        }
    }
}
