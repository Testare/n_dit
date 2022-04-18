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
                // TODO Apply Enemy Actions should not be in the animation logic: At very least, in the AI logic
                let mut enemy_actions_vec_clone = enemy_actions_vec.clone();
                let node = game.node_mut().ok_or_else(|| {
                    "Enemy AI animation shouldn't occur when there is no Node".to_owned()
                })?; // TODO stabalize on String::from(..) vs .to_owned() vs .to_string()
                if let Some(enemy_action) = enemy_actions_vec_clone.pop() {
                    match enemy_action {
                        EnemyAiAction::PerformNoAction => {
                            debug!("Active sprite deactivating with no action");
                            node.deactivate_sprite();
                        }
                        EnemyAiAction::MoveSprite(dir) => {
                            debug!("Sprite movement occured in dir {:?}", dir);
                            node.move_active_sprite(&[dir])?;
                        }
                        EnemyAiAction::ActivateSprite(sprite_key) => {
                            debug!("Sprite key activated {:?}", sprite_key);

                            node.activate_sprite(sprite_key);
                        }
                        EnemyAiAction::PerformAction(action_index, pt) => {
                            debug!(
                                "Performing sprite action {:?} (index {:?}) on point {:?}",
                                node.with_active_sprite::<_, &str, _>(|sprite| {
                                    sprite
                                        .actions()
                                        .get(action_index)
                                        .map(|action| *(*action).name())
                                }),
                                action_index,
                                pt
                            );
                            node.perform_sprite_action(action_index, pt);
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
