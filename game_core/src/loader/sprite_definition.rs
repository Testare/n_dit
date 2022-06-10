use serde::{Deserialize, Serialize};

use crate::{Pickup, Point, Team, Metadata};
use crate::Asset;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "type")]
pub enum SpriteDef {
    Pickup {
        #[serde(flatten)]
        pickup: Pickup,
        point: Point,
    },
    Curio {
        #[serde(default, skip_serializing_if="Metadata::is_empty")]
        metadata: Metadata,
        #[serde(default, skip_serializing_if="Option::is_none")]
        nickname: Option<String>,
        team: Team,
        points: Vec<Point>,
        #[serde(flatten)]
        def: CardInstanceDefAlternative,
    },
    AccessPoint {
        point: Point,
    },
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(untagged)]
pub enum CardInstanceDefAlternative {
    FromTemplate {
        card: String,
    },
    Custom {
        actions: Vec<String>,
        display: String,
        max_size: usize,
        speed: usize,
        name: String,
    },
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct CardDef {
    pub actions: Vec<String>,
    pub display: String,
    pub max_size: usize,
    pub name: String,
    pub speed: usize,
    // mind: Mind // Save for post-nightfall
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct CardDefUnnamed {
    pub actions: Vec<String>,
    pub display: String,
    pub max_size: usize,
    pub speed: usize,
    // mind: Mind // Save for post-nightfall
}

impl Asset for CardDef {
    const SUB_EXTENSION: &'static str = "cards";
    type UnnamedAsset = CardDefUnnamed;

    fn with_name(unnamed: Self::UnnamedAsset, name: &str) -> Self {
        CardDef {
            actions: unnamed.actions,
            display: unnamed.display,
            max_size: unnamed.max_size,
            name: name.to_string(),
            speed: unnamed.speed,
        }
    }
}
