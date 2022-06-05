use super::{CurioAction, CurioActionGenre, SACondition, SAEffect, Target};
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
    pub fn unwrap(&self) -> &'static CurioAction {
        match self {
            StandardCurioAction::Brutus => &BRUTUS,
            StandardCurioAction::Bite => &BITE,
            StandardCurioAction::Fiddle => &FIDDLE,
        }
    }
}

impl Deref for StandardCurioAction {
    type Target = CurioAction;

    fn deref(&self) -> &Self::Target {
        self.unwrap()
    }
}

impl From<StandardCurioAction> for &'static CurioAction {
    fn from(standard_curio_action: StandardCurioAction) -> Self {
        standard_curio_action.unwrap()
    }
}

lazy_static! {
    static ref BRUTUS: CurioAction = CurioAction {
        name: "Brutus".to_string(),
        genre: CurioActionGenre::Attack,
        range: NonZeroUsize::new(2),
        effect: SAEffect::DealDamage(2),
        targets: vec![Target::Ally],
        conditions: Vec::new()
    };
    static ref BITE: CurioAction = CurioAction {
        name: "Bite".to_string(),
        genre: CurioActionGenre::Attack,
        range: NonZeroUsize::new(1),
        effect: SAEffect::DealDamage(2),
        targets: vec![Target::Enemy],
        conditions: Vec::new()
    };
    static ref FIDDLE: CurioAction = CurioAction {
        name: "Fiddle".to_string(),
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
