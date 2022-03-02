use super::super::{Direction, Node};
use super::EnemyAiAction;

pub(super) fn generate_enemy_ai_actions(
    _node: Node,
    team_sprites: Vec<usize>,
) -> Vec<EnemyAiAction> {
    // Currently just move all sprites to the right
    let mut actions = Vec::new();
    for sprite_key in team_sprites {
        actions.push(EnemyAiAction::PerformNoAction);
        actions.push(EnemyAiAction::MoveSprite(Direction::East));
        actions.push(EnemyAiAction::ActivateSprite(sprite_key));
    }
    actions
}
