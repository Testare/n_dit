use super::{Direction, Node, Point};

mod simple_ai;

pub enum EnemyAiAction {
    ActivateSprite(usize),
    MoveSprite(Direction),
    PerformAction(usize, Point),
    PerformNoAction,
}

#[derive(Clone, Debug)]
pub enum EnemyAi {
    SimpleAi
}

impl EnemyAi {
    pub fn generate_enemy_ai_actions(&self, node: &Node) -> Vec<EnemyAiAction> {
        let node_destructible = node.clone();
        match node.enemy_ai() {
            SimpleAi => simple_ai::generate_enemy_ai_actions(node)
        }
    }
}