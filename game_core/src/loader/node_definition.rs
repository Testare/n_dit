use serde::{Deserialize, Serialize};

use super::sprite_definition::SpriteDef;
use crate::Asset;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct NodeDef {
    grid_shape: String,
    sprites: Vec<SpriteDef>,
}

impl Asset for NodeDef {
    const SUB_EXTENSION: &'static str = "nodes";
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
                        movement_speed: 2,
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
