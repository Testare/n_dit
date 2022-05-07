use super::{GameState};

#[derive(Debug, Clone)]
pub enum Animation {
    EnemyActions, // Might need to be rethought
}

impl Animation {
    pub fn next(game: &mut GameState) -> Result<(), String> {
        match game.animation() {
            None => Err(
                "next was called for animation, but there is no animation to be called".to_owned(),
            ),
            Some(Animation::EnemyActions) => {
                unimplemented!("Animations not implemented");
            }
        }
    }
}
