use serde::{Deserialize, Serialize};

use super::sprite_definition::SpriteDef;
use crate::Asset;

use super::{LoadingError, CardInstanceDefAlternative};
use crate::{Curio, CurioAction, CardDef, AssetDictionary, Node, GridMap, Sprite, };

#[derive(Debug, Deserialize, Serialize)]
pub struct NodeDefUnnamed {
    grid_shape: String,
    sprites: Vec<SpriteDef>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct NodeDef {
    grid_shape: String,
    name: String,
    sprites: Vec<SpriteDef>,
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

pub fn node_from_def(def: &NodeDef, curio_templates: AssetDictionary<CardDef>, action_dictionary: AssetDictionary<CurioAction>) -> Result<Node, LoadingError> {
    let mut node = Node::from(GridMap::from_shape_string(def.grid_shape.as_str())?);
    node.add_action_dictionary(action_dictionary);
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
                def
            } => {
                // let nickname: String = nickname.clone().unwrap_or_else(||"Nameless".to_string());
                let mut builder = Curio::builder();
                let builder = builder
                    .team(*team)
                    .metadata(metadata.clone());

                let builder = match def {
                    CardInstanceDefAlternative::FromTemplate { card } => {
                        let template = curio_templates.get(card)
                            .ok_or_else(||LoadingError::MissingAsset(CardDef::SUB_EXTENSION, card.clone()))?;
                        builder.actions(&template.actions)
                            .speed(template.speed)
                            .max_size(template.max_size)
                            .display(&template.display)
                            .name(nickname.as_ref().unwrap_or(&template.name).clone())
                    }, 
                    CardInstanceDefAlternative::Custom {
                        name, 
                        actions,
                        speed,
                        max_size,
                        display
                    } => {
                        builder.actions(actions)
                            .speed(*speed)
                            .max_size(*max_size)
                            .display(display)
                            .name(nickname.as_ref().unwrap_or(name).clone())
                    }
                };
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
