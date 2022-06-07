use serde::{Deserialize, Serialize};

use crate::{Pickup, Point, Team};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "type")]
pub enum SpriteDef {
    Pickup {
        #[serde(flatten)]
        pickup: Pickup,
        point: Point,
    },
    Curio {
        nickname: Option<String>,
        team: Team,
        points: Vec<Point>,
        #[serde(flatten)]
        def: CurioDefAlternative,
    },
    AccessPoint {
        point: Point,
    },
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(untagged)]
pub enum CurioDefAlternative {
    FromTemplate {
        template_name: String,
    },
    CustomDef {
        actions: Vec<String>,
        movement_speed: usize,
        max_size: usize,
        display: String,
    },
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct CurioTemplateDef {
    actions: Vec<String>,
    movement_speed: usize,
    max_size: usize,
    display: String,
    // mind: Mind // Save for post-nightfall
}
