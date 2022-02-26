use super::super::{Direction, Node, Team};
use super::EnemyAiAction;

pub(super) fn generate_enemy_ai_actions(node: Node) -> Vec<EnemyAiAction> {
    // Currently a stupid AI that will always try to move one of its piece to the right each time
    let team_sprites = node.filtered_sprite_keys(|_, sprite| sprite.team() == Team::EnemyTeam);
    let sprite_key_1 = team_sprites[0];
    vec![
        EnemyAiAction::ActivateSprite(sprite_key_1),
        EnemyAiAction::MoveSprite(Direction::East),
        EnemyAiAction::PerformNoAction,
    ]
}
