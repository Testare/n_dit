use std::sync::Arc;
use serde::{Deserialize, Serialize};

use crate::Asset;
use crate::{Pickup, Point, Metadata, Team};
use super::{CardDef, ActionDef};

use crate::error::{LoadingError};
use crate::{Curio, AssetDictionary, Node, GridMap, Sprite};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct NodeDef {
    grid_shape: String,
    name: String,
    sprites: Vec<SpriteDef>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct NodeDefUnnamed {
    grid_shape: String,
    sprites: Vec<SpriteDef>,
}

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
        card: CardRef,
    },
    AccessPoint {
        point: Point,
    },
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(untagged)]
pub enum CardRef {
    FromAsset(String),
    Custom(CardDef),
}

impl Asset for NodeDef {
    const SUB_EXTENSION: &'static str = "nodes";
    type UnnamedAsset = NodeDefUnnamed;

    fn with_name(unnamed: Self::UnnamedAsset, name: &str) -> Self {
        NodeDef {
            grid_shape: unnamed.grid_shape,
            name: name.to_string(),
            sprites: unnamed.sprites,
        }
    }
}

//
pub fn node_from_def(def: &NodeDef, card_dict: AssetDictionary<CardDef>, action_dictionary: AssetDictionary<ActionDef>) -> Result<Node, LoadingError> {
    let mut node = Node::from(GridMap::from_shape_string(def.grid_shape.as_str())?);
    node.add_action_dictionary(action_dictionary);
    node.add_card_dictionary(card_dict.clone());

    for sprite_def in def.sprites.iter() {
        match sprite_def {
            SpriteDef::Pickup { pickup, point } => {
                node.add_sprite(*point, pickup.clone().to_sprite());
            },
            SpriteDef::AccessPoint { point } => {
                node.add_sprite(*point, Sprite::AccessPoint);
            },
            SpriteDef::Curio {
                nickname,
                metadata,
                team,
                points,
                card
            } => {
                let card_ref = match card {
                    CardRef::FromAsset ( card ) => {
                        card_dict.get(card)
                            .ok_or_else(||LoadingError::MissingAsset(CardDef::SUB_EXTENSION, card.clone()))?
                    }, 
                    CardRef::Custom ( card ) => Arc::new(card.clone())
                };
                let mut builder = Curio::builder();
                let builder = builder
                    .team(*team)
                    .metadata(metadata.clone())
                    .actions(&card_ref.actions)
                    .speed(card_ref.speed)
                    .max_size(card_ref.max_size)
                    .display(card_ref.display.clone())
                    .name(nickname.as_ref().unwrap_or(&card_ref.name).clone());
                    // TODO Error handling for build error here
                node.add_curio(builder.build().unwrap(), points.clone()).unwrap();
            }
        }
    }
    Ok(node)
}

#[cfg(test)]
mod test {
    use super::super::sprite_definition::{CurioInstanceDefAlternative, SpriteDef};
    use super::{NodeDef};
    use crate::{Pickup, Team};

    #[test]
    fn node_def_sede_test() {
        let node = NodeDef {
            grid_shape: "EwALACCAAz7447/vP/7x+AABPh7/+O/7jz/4gAMIAA==".to_string(),
            sprites: vec![
                SpriteDef::AccessPoint { point: (0, 1) },
                SpriteDef::Pickup {
                    pickup: Pickup::Mon(400),
                    point: (0, 2),
                },
                SpriteDef::Curio {
                    nickname: Some("Coleslaw".to_string()),
                    team: Team::EnemyTeam,
                    points: vec![(0, 3), (0, 4)],
                    def: CurioInstanceDefAlternative::FromTemplate {
                        template_name: "Nelson".to_string(),
                    },
                },
                SpriteDef::Curio {
                    nickname: Some("Grimothy".to_string()),
                    team: Team::EnemyTeam,
                    points: vec![(0, 5)],
                    def: CurioInstanceDefAlternative::CustomDef {
                        actions: vec!["Bite".to_string()],
                        speed: 2,
                        max_size: 1,
                        display: "][".to_string(),
                    },
                },
            ],
        };
        let json = serde_json::to_string_pretty(&node).unwrap();
        println!("{}", json);
        assert_eq!(
            serde_json::from_str::<NodeDef>(json.as_str()).unwrap(),
            node
        )
    }
}
