use super::{SACondition, SAEffect, SpriteAction, SpriteActionGenre, Target};
use std::num::NonZeroUsize;
use std::ops::Deref;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum StandardSpriteAction {
    Brutus,
    Bite,
    Fiddle,
}

impl StandardSpriteAction {
    pub fn unwrap(&self) -> &'static SpriteAction<'static> {
        match self {
            StandardSpriteAction::Brutus => &BRUTUS,
            StandardSpriteAction::Bite => &BITE,
            StandardSpriteAction::Fiddle => &FIDDLE,
        }
    }
}

impl Deref for StandardSpriteAction {
    type Target = SpriteAction<'static>;

    fn deref(&self) -> &Self::Target {
        self.unwrap()
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
        targets: vec![Target::Ally],
        conditions: Vec::new()
    };
    static ref BITE: SpriteAction<'static> = SpriteAction {
        name: "Bite",
        genre: SpriteActionGenre::Attack,
        range: NonZeroUsize::new(1),
        effect: SAEffect::DealDamage(2),
        targets: vec![Target::Enemy],
        conditions: Vec::new()
    };
    static ref FIDDLE: SpriteAction<'static> = SpriteAction {
        name: "Fiddle",
        genre: SpriteActionGenre::Support,
        range: NonZeroUsize::new(2),
        effect: SAEffect::IncreaseMaxSize {
            amount: 1,
            bound: NonZeroUsize::new(4)
        },
        targets: vec![Target::Ally],
        conditions: vec![SACondition::TargetMaxSize(1..=4)],
    };
}
