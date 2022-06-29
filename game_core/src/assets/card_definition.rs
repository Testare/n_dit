use serde::{Deserialize, Serialize};

use crate::Asset;


#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct CardDef {
    pub actions: Vec<String>,
    pub description: String,
    pub display: String,
    pub max_size: usize,
    pub name: String,
    pub speed: usize,
    // mind: Mind // Save for post-nightfall
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct CardDefUnnamed {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub actions: Vec<String>,
    pub description: String,
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
            description: unnamed.description,
            display: unnamed.display,
            max_size: unnamed.max_size,
            name: name.to_string(),
            speed: unnamed.speed,
        }
    }
}