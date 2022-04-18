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

#[derive(Copy, Clone, Debug)]
pub enum EnemyAi {
    Simple,
}

impl EnemyAi {
    pub fn generate_enemy_ai_actions<C: FnMut(EnemyAiAction)>(&self, node: Node, mut collect: C) {
        let keys = node.sprite_keys_for_team(Team::EnemyTeam);
        match node.enemy_ai() {
            EnemyAi::Simple => simple_ai::generate_enemy_ai_actions(
                node,
                keys,
                collect
            ),
        }
    }

    pub fn generate_animation(&self, node: &Node) -> Animation {
        let mut vec: Vec<EnemyAiAction> = Vec::new();
        self.generate_enemy_ai_actions(node.clone(), |action| {vec.push(action)});
        Animation::EnemyActions(vec.into_iter().rev().collect())
    }
}
