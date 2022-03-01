use super::{EnemyAiAction, GameState};
use log::debug;

#[derive(Debug, Clone)]
pub enum Animation {
    EnemyActions(Vec<EnemyAiAction>),
}

impl Animation {
    pub fn next(game: &mut GameState) -> Result<(), String> {
        match game.animation() {
            None => Err(
                "next was called for animation, but there is no animation to be called".to_owned(),
            ),
            Some(Animation::EnemyActions(enemy_actions_vec)) => {
                let mut enemy_actions_vec_clone = enemy_actions_vec.clone();
                let node = game
                    .node_mut()
                    .ok_or("Enemy AI animation shouldn't occur when there is no Node".to_owned())?;
                if let Some(enemy_action) = enemy_actions_vec_clone.pop() {
                    match enemy_action {
                        EnemyAiAction::PerformNoAction => {
                            debug!("Active sprite deactivating with no action");
                            node.deactivate_sprite();
                        }
                        EnemyAiAction::MoveSprite(dir) => {
                            debug!("Sprite movement occured in dir {:?}", dir);
                            node.move_active_sprite(vec![dir])?;
                        }
                        EnemyAiAction::ActivateSprite(sprite_key) => {
                            debug!("Sprite key activated {:?}", sprite_key);

                            node.activate_sprite(sprite_key);
                        }
                        _ => {
                            unimplemented!(
                                "Enemy action {:?} has not been implemented yet",
                                enemy_action
                            );
                        }
                    }
                    game.set_animation(Animation::EnemyActions(enemy_actions_vec_clone))
                } else {
                    node.change_active_team();
                    game.set_animation(None);
                }
                Ok(())
            }
        }
    }
}
