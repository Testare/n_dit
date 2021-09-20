use super::{SAEffect, SpriteAction, SpriteActionGenre, Target};
use std::num::NonZeroUsize;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StandardSpriteAction {
    Brutus,
    Bite,
    Fiddle,
}

impl StandardSpriteAction {
    pub fn unwrap(&self) -> &'static SpriteAction<'static> {
        match self {
            StandardSpriteAction::Brutus => &BRUTUS,
            _ => unimplemented!("Not implemented yet"),
        }
    }
}

impl From<StandardSpriteAction> for &'static SpriteAction<'static> {
    fn from(standard_sprite_action: StandardSpriteAction) -> Self {
        standard_sprite_action.unwrap()
    }
}

lazy_static! {
    static ref BRUTUS: SpriteAction<'static> = SpriteAction {
        name: "Brutus",
        genre: SpriteActionGenre::Attack,
        range: NonZeroUsize::new(2),
        effect: SAEffect::DealDamage(2),
        targets: vec![Target::Ally, Target::Enemy],
        conditions: Vec::new()
    };
}
