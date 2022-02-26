use super::{Direction, Node, Point, Team};

mod simple_ai;

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
    pub fn generate_enemy_ai_actions(&self, node: &Node) -> Vec<EnemyAiAction> {
        let node_destructible = node.clone();
        match node.enemy_ai() {
            EnemyAi::Simple => simple_ai::generate_enemy_ai_actions(node_destructible),
        }
    }
}

fn find_sprites_on_team(node: &Node) -> Vec<usize> {
    node.filtered_sprite_keys(|_, sprite| sprite.team() == Team::EnemyTeam)
}
