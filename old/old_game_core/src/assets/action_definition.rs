use std::num::NonZeroUsize;
use std::ops::RangeInclusive;

use getset::{CopyGetters, Getters};
use serde::{Deserialize, Serialize};

use crate::{Asset, Sprite};

#[derive(Debug, Deserialize, Serialize)]
pub struct ActionDefUnnamed {
    pub genre: CurioActionGenre,
    pub range: Option<NonZeroUsize>,
    pub effect: ActionEffect,
    pub targets: Vec<ActionTarget>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub conditions: Vec<ActionCondition>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cost: Option<NonZeroUsize>,
}

#[derive(Clone, Debug, CopyGetters, Getters, Deserialize, Serialize)]
pub struct ActionDef {
    #[get = "pub"]
    name: String,
    #[get_copy = "pub"]
    genre: CurioActionGenre,
    #[get_copy = "pub"]
    range: Option<NonZeroUsize>,
    #[get = "pub"]
    effect: ActionEffect,
    #[get = "pub"]
    targets: Vec<ActionTarget>,
    #[get = "pub"]
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    conditions: Vec<ActionCondition>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    cost: Option<NonZeroUsize>,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub enum CurioActionGenre {
    Attack = 0,
    Support = 1,
    Special = 2,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub enum ActionEffect {
    DealDamage(usize),
    IncreaseMaxSize {
        amount: usize,
        bound: Option<NonZeroUsize>,
    },
    DecreaseMaxSize {
        amount: usize,
        bound: Option<NonZeroUsize>,
    },
    IncreaseSpeed {
        amount: usize,
        bound: Option<NonZeroUsize>,
    },
    DecreaseSpeed {
        amount: usize,
        bound: Option<NonZeroUsize>,
    },
    Recover {
        amount: usize,
        bound: Option<NonZeroUsize>,
    },
    _Create {
        sprite: Sprite,
    },
    OpenSquare,
    CloseSquare,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub enum ActionCondition {
    Size(RangeInclusive<usize>),
    TargetSize(RangeInclusive<usize>),
    TargetMaxSize(RangeInclusive<usize>),
    _Uses(usize),
    _UsesPerTarget(usize),
}

// TODO enumset?
#[derive(Clone, Copy, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub enum ActionTarget {
    Ally = 0,
    // Area,
    ClosedSquare,
    EmptySquare,
    Enemy,
    _Itself,
}

impl Asset for ActionDef {
    const SUB_EXTENSION: &'static str = "actions";
    type UnnamedAsset = ActionDefUnnamed;

    fn with_name(unnamed: Self::UnnamedAsset, name: &str) -> Self {
        ActionDef {
            name: name.to_string(),
            genre: unnamed.genre,
            range: unnamed.range,
            effect: unnamed.effect,
            targets: unnamed.targets,
            conditions: unnamed.conditions,
            cost: unnamed.cost,
        }
    }
}
