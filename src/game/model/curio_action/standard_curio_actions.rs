use super::{SACondition, SAEffect, CurioAction, CurioActionGenre, Target};
use serde::{Deserialize, Serialize};
use std::num::NonZeroUsize;
use std::ops::Deref;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum StandardCurioAction {
    Brutus,
    Bite,
    Fiddle,
}

impl StandardCurioAction {
    pub fn unwrap(&self) -> &'static CurioAction<'static> {
        match self {
            StandardCurioAction::Brutus => &BRUTUS,
            StandardCurioAction::Bite => &BITE,
            StandardCurioAction::Fiddle => &FIDDLE,
        }
    }
}

impl Deref for StandardCurioAction {
    type Target = CurioAction<'static>;

    fn deref(&self) -> &Self::Target {
        self.unwrap()
    }
}

impl From<StandardCurioAction> for &'static CurioAction<'static> {
    fn from(standard_curio_action: StandardCurioAction) -> Self {
        standard_curio_action.unwrap()
    }
}

lazy_static! {
    static ref BRUTUS: CurioAction<'static> = CurioAction {
        name: "Brutus",
        genre: CurioActionGenre::Attack,
        range: NonZeroUsize::new(2),
        effect: SAEffect::DealDamage(2),
        targets: vec![Target::Ally],
        conditions: Vec::new()
    };
    static ref BITE: CurioAction<'static> = CurioAction {
        name: "Bite",
        genre: CurioActionGenre::Attack,
        range: NonZeroUsize::new(1),
        effect: SAEffect::DealDamage(2),
        targets: vec![Target::Enemy],
        conditions: Vec::new()
    };
    static ref FIDDLE: CurioAction<'static> = CurioAction {
        name: "Fiddle",
        genre: CurioActionGenre::Support,
        range: NonZeroUsize::new(2),
        effect: SAEffect::IncreaseMaxSize {
            amount: 1,
            bound: NonZeroUsize::new(5)
        },
        targets: vec![Target::Ally],
        conditions: vec![SACondition::TargetMaxSize(1..=4)],
    };
}
