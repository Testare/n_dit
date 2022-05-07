use super::{GameAction, Node, Team};

mod pathfinding;
mod simple_ai;

#[derive(Copy, Clone, Debug)]
pub enum EnemyAi {
    Simple,
}

impl EnemyAi {
    pub fn generate_enemy_ai_actions<C: FnMut(GameAction)>(&self, node: Node, collect: C) {
        let keys = node.sprite_keys_for_team(Team::EnemyTeam);
        match node.enemy_ai() {
            EnemyAi::Simple => simple_ai::generate_enemy_ai_actions(node, keys, collect),
        }
    }
}
