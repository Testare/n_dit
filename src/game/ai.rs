use super::{Animation, Direction, Node, Point, Team};

mod pathfinding;
mod simple_ai;

#[derive(Clone, Debug)]
pub enum EnemyAiAction {
    ActivateSprite(usize),
    MoveSprite(Direction),
    PerformAction(usize, Point),
    PerformNoAction,
}

#[derive(Clone, Debug)]
pub enum EnemyAi {
    Simple,
}

impl EnemyAi {
    fn generate_enemy_ai_actions(&self, node: &Node) -> Vec<EnemyAiAction> {
        let node_destructible = node.clone();
        match node.enemy_ai() {
            EnemyAi::Simple => simple_ai::generate_enemy_ai_actions(
                node_destructible,
                node.sprite_keys_for_team(Team::EnemyTeam),
            ),
        }
    }

    pub fn generate_animation(&self, node: &Node) -> Animation {
        Animation::EnemyActions(self.generate_enemy_ai_actions(node))
    }
}
