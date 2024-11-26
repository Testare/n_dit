use serde::{Deserialize, Serialize};

use crate::prelude::*;

#[derive(Component, Debug, Default, Deserialize, Serialize)]
pub struct PlayerConfiguration {
    pub node: Option<NodeConfiguration>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct NodeConfiguration {
    #[serde(default = "bool_true")]
    pub end_turn_after_all_pieces_tap: bool,
}

fn bool_true() -> bool {
    true
}
