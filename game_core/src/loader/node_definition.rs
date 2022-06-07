use bitvec::vec::BitVec;
use serde::{Deserialize, Serialize};

use super::sprite_definition::SpriteDef;
use crate::GridMap;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct GridMapDef {
    width: usize,
    height: usize,
    shape: String,
}

impl GridMapDef {
    fn to_base_grid_map<T>(&self) -> GridMap<T> {
        let bits: Vec<u8> = base64::decode(self.shape.as_str()).unwrap();
        let bitvec = BitVec::from_vec(bits);
        GridMap::from_bitslice(self.width, self.height, bitvec.as_bitslice())
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
struct NodeDef {
    grid: GridMapDef,
    sprites: Vec<SpriteDef>,
}

#[cfg(test)]
mod test {
    use super::super::sprite_definition::{CurioDefAlternative, SpriteDef};
    use super::{GridMapDef, NodeDef};
    use crate::{GridMap, Pickup, Team};

    #[test]
    fn node_def_sede_test() {
        let node = NodeDef {
            grid: GridMapDef {
                width: 10,
                height: 5,
                shape: "abdefbdafcd082".to_string(),
            },
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
                    def: CurioDefAlternative::FromTemplate {
                        template_name: "Nelson".to_string(),
                    },
                },
                SpriteDef::Curio {
                    nickname: Some("Grimothy".to_string()),
                    team: Team::EnemyTeam,
                    points: vec![(0, 5)],
                    def: CurioDefAlternative::CustomDef {
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

    #[test]
    fn grid_def_into_map_test() {
        let expected_grid: GridMap<()> = GridMap::from(vec![
            vec![
                false, false, false, false, false, true, false, false, false, false, false,
            ],
            vec![
                false, false, false, false, true, true, true, false, false, false, false,
            ],
            vec![
                false, false, false, true, true, true, true, true, false, false, false,
            ],
            vec![
                false, false, true, true, true, true, true, true, true, false, false,
            ],
            vec![
                false, true, true, true, true, true, true, true, true, true, false,
            ],
            vec![
                true, true, true, true, true, false, true, true, true, true, true,
            ],
            vec![
                true, true, true, true, false, false, false, true, true, true, true,
            ],
            vec![
                true, true, true, true, false, false, false, true, true, true, true,
            ],
            vec![
                false, false, false, true, true, true, true, true, false, false, false,
            ],
            vec![
                false, false, false, false, false, true, false, false, false, false, false,
            ],
            vec![
                false, false, false, true, true, true, true, true, false, false, false,
            ],
            vec![
                true, true, true, true, false, false, false, true, true, true, true,
            ],
            vec![
                true, true, true, true, false, false, false, true, true, true, true,
            ],
            vec![
                true, true, true, true, true, false, true, true, true, true, true,
            ],
            vec![
                false, true, true, true, true, true, true, true, true, true, false,
            ],
            vec![
                false, false, true, true, true, true, true, true, true, false, false,
            ],
            vec![
                false, false, false, true, true, true, true, true, false, false, false,
            ],
            vec![
                false, false, false, false, true, true, true, false, false, false, false,
            ],
            vec![
                false, false, false, false, false, true, false, false, false, false, false,
            ],
        ]);
        let grid_def = GridMapDef {
            width: 19,
            height: 11,
            shape: "IIADPvjjv+8//vH4AAE+Hv/47/uPP/iAAwgA".to_string(),
        };
        let grid_map = grid_def.to_base_grid_map::<()>();

        assert_eq!(expected_grid, grid_map);
    }
}

/*

How to handle extra 0's causing
DEF  GridMap { width: 19, height: 11, next_id: 2, entries: {}, grid: [[None, None, None, None, None, Some(Square { item: None, next: None, location: (0, 5) }), None, None, None, None, None], [None, None, None, None, Some(Square { item: None, next: None, location: (1, 4) }), Some(Square { item: None, next: None, location: (1, 5) }), Some(Square { item: None, next: None, location: (1, 6) }), None, None, None, None], [None, None, None, Some(Square { item: None, next: None, location: (2, 3) }), Some(Square { item: None, next: None, location: (2, 4) }), Some(Square { item: None, next: None, location: (2, 5) }), Some(Square { item: None, next: None, location: (2, 6) }), Some(Square { item: None, next: None, location: (2, 7) }), None, None, None], [None, None, Some(Square { item: None, next: None, location: (3, 2) }), Some(Square { item: None, next: None, location: (3, 3) }), Some(Square { item: None, next: None, location: (3, 4) }), Some(Square { item: None, next: None, location: (3, 5) }), Some(Square { item: None, next: None, location: (3, 6) }), Some(Square { item: None, next: None, location: (3, 7) }), Some(Square { item: None, next: None, location: (3, 8) }), None, None], [None, Some(Square { item: None, next: None, location: (4, 1) }), Some(Square { item: None, next: None, location: (4, 2) }), Some(Square { item: None, next: None, location: (4, 3) }), Some(Square { item: None, next: None, location: (4, 4) }), Some(Square { item: None, next: None, location: (4, 5) }), Some(Square { item: None, next: None, location: (4, 6) }), Some(Square { item: None, next: None, location: (4, 7) }), Some(Square { item: None, next: None, location: (4, 8) }), Some(Square { item: None, next: None, location: (4, 9) }), None], [Some(Square { item: None, next: None, location: (5, 0) }), Some(Square { item: None, next: None, location: (5, 1) }), Some(Square { item: None, next: None, location: (5, 2) }), Some(Square { item: None, next: None, location: (5, 3) }), Some(Square { item: None, next: None, location: (5, 4) }), None, Some(Square { item: None, next: None, location: (5, 6) }), Some(Square { item: None, next: None, location: (5, 7) }), Some(Square { item: None, next: None, location: (5, 8) }), Some(Square { item: None, next: None, location: (5, 9) }), Some(Square { item: None, next: None, location: (5, 10) })], [Some(Square { item: None, next: None, location: (6, 0) }), Some(Square { item: None, next: None, location: (6, 1) }), Some(Square { item: None, next: None, location: (6, 2) }), Some(Square { item: None, next: None, location: (6, 3) }), None, None, None, Some(Square { item: None, next: None, location: (6, 7) }), Some(Square { item: None, next: None, location: (6, 8) }), Some(Square { item: None, next: None, location: (6, 9) }), Some(Square { item: None, next: None, location: (6, 10) })], [Some(Square { item: None, next: None, location: (7, 0) }), Some(Square { item: None, next: None, location: (7, 1) }), Some(Square { item: None, next: None, location: (7, 2) }), Some(Square { item: None, next: None, location: (7, 3) }), None, None, None, Some(Square { item: None, next: None, location: (7, 7) }), Some(Square { item: None, next: None, location: (7, 8) }), Some(Square { item: None, next: None, location: (7, 9) }), Some(Square { item: None, next: None, location: (7, 10) })], [None, None, None, Some(Square { item: None, next: None, location: (8, 3) }), Some(Square { item: None, next: None, location: (8, 4) }), Some(Square { item: None, next: None, location: (8, 5) }), Some(Square { item: None, next: None, location: (8, 6) }), Some(Square { item: None, next: None, location: (8, 7) }), None, None, None], [None, None, None, None, None, Some(Square { item: None, next: None, location: (9, 5) }), None, None, None, None, None], [None, None, None, Some(Square { item: None, next: None, location: (10, 3) }), Some(Square { item: None, next: None, location: (10, 4) }), Some(Square { item: None, next: None, location: (10, 5) }), Some(Square { item: None, next: None, location: (10, 6) }), Some(Square { item: None, next: None, location: (10, 7) }), None, None, None], [Some(Square { item: None, next: None, location: (11, 0) }), Some(Square { item: None, next: None, location: (11, 1) }), Some(Square { item: None, next: None, location: (11, 2) }), Some(Square { item: None, next: None, location: (11, 3) }), None, None, None, Some(Square { item: None, next: None, location: (11, 7) }), Some(Square { item: None, next: None, location: (11, 8) }), Some(Square { item: None, next: None, location: (11, 9) }), Some(Square { item: None, next: None, location: (11, 10) })], [Some(Square { item: None, next: None, location: (12, 0) }), Some(Square { item: None, next: None, location: (12, 1) }), Some(Square { item: None, next: None, location: (12, 2) }), Some(Square { item: None, next: None, location: (12, 3) }), None, None, None, Some(Square { item: None, next: None, location: (12, 7) }), Some(Square { item: None, next: None, location: (12, 8) }), Some(Square { item: None, next: None, location: (12, 9) }), Some(Square { item: None, next: None, location: (12, 10) })], [Some(Square { item: None, next: None, location: (13, 0) }), Some(Square { item: None, next: None, location: (13, 1) }), Some(Square { item: None, next: None, location: (13, 2) }), Some(Square { item: None, next: None, location: (13, 3) }), Some(Square { item: None, next: None, location: (13, 4) }), None, Some(Square { item: None, next: None, location: (13, 6) }), Some(Square { item: None, next: None, location: (13, 7) }), Some(Square { item: None, next: None, location: (13, 8) }), Some(Square { item: None, next: None, location: (13, 9) }), Some(Square { item: None, next: None, location: (13, 10) })], [None, Some(Square { item: None, next: None, location: (14, 1) }), Some(Square { item: None, next: None, location: (14, 2) }), Some(Square { item: None, next: None, location: (14, 3) }), Some(Square { item: None, next: None, location: (14, 4) }), Some(Square { item: None, next: None, location: (14, 5) }), Some(Square { item: None, next: None, location: (14, 6) }), Some(Square { item: None, next: None, location: (14, 7) }), Some(Square { item: None, next: None, location: (14, 8) }), Some(Square { item: None, next: None, location: (14, 9) }), None], [None, None, Some(Square { item: None, next: None, location: (15, 2) }), Some(Square { item: None, next: None, location: (15, 3) }), Some(Square { item: None, next: None, location: (15, 4) }), Some(Square { item: None, next: None, location: (15, 5) }), Some(Square { item: None, next: None, location: (15, 6) }), Some(Square { item: None, next: None, location: (15, 7) }), Some(Square { item: None, next: None, location: (15, 8) }), None, None], [None, None, None, Some(Square { item: None, next: None, location: (16, 3) }), Some(Square { item: None, next: None, location: (16, 4) }), Some(Square { item: None, next: None, location: (16, 5) }), Some(Square { item: None, next: None, location: (16, 6) }), Some(Square { item: None, next: None, location: (16, 7) }), None, None, None], [None, None, None, None, Some(Square { item: None, next: None, location: (17, 4) }), Some(Square { item: None, next: None, location: (17, 5) }), Some(Square { item: None, next: None, location: (17, 6) }), None, None, None, None], [None, None, None, None, None, Some(Square { item: None, next: None, location: (18, 5) }), None, None, None, None, None], [None, None, None, None, None, None, None]] }
NODE GridMap { width: 19, height: 11, next_id: 2, entries: {}, grid: [[None, None, None, None, None, Some(Square { item: None, next: None, location: (0, 5) }), None, None, None, None, None], [None, None, None, None, Some(Square { item: None, next: None, location: (1, 4) }), Some(Square { item: None, next: None, location: (1, 5) }), Some(Square { item: None, next: None, location: (1, 6) }), None, None, None, None], [None, None, None, Some(Square { item: None, next: None, location: (2, 3) }), Some(Square { item: None, next: None, location: (2, 4) }), Some(Square { item: None, next: None, location: (2, 5) }), Some(Square { item: None, next: None, location: (2, 6) }), Some(Square { item: None, next: None, location: (2, 7) }), None, None, None], [None, None, Some(Square { item: None, next: None, location: (3, 2) }), Some(Square { item: None, next: None, location: (3, 3) }), Some(Square { item: None, next: None, location: (3, 4) }), Some(Square { item: None, next: None, location: (3, 5) }), Some(Square { item: None, next: None, location: (3, 6) }), Some(Square { item: None, next: None, location: (3, 7) }), Some(Square { item: None, next: None, location: (3, 8) }), None, None], [None, Some(Square { item: None, next: None, location: (4, 1) }), Some(Square { item: None, next: None, location: (4, 2) }), Some(Square { item: None, next: None, location: (4, 3) }), Some(Square { item: None, next: None, location: (4, 4) }), Some(Square { item: None, next: None, location: (4, 5) }), Some(Square { item: None, next: None, location: (4, 6) }), Some(Square { item: None, next: None, location: (4, 7) }), Some(Square { item: None, next: None, location: (4, 8) }), Some(Square { item: None, next: None, location: (4, 9) }), None], [Some(Square { item: None, next: None, location: (5, 0) }), Some(Square { item: None, next: None, location: (5, 1) }), Some(Square { item: None, next: None, location: (5, 2) }), Some(Square { item: None, next: None, location: (5, 3) }), Some(Square { item: None, next: None, location: (5, 4) }), None, Some(Square { item: None, next: None, location: (5, 6) }), Some(Square { item: None, next: None, location: (5, 7) }), Some(Square { item: None, next: None, location: (5, 8) }), Some(Square { item: None, next: None, location: (5, 9) }), Some(Square { item: None, next: None, location: (5, 10) })], [Some(Square { item: None, next: None, location: (6, 0) }), Some(Square { item: None, next: None, location: (6, 1) }), Some(Square { item: None, next: None, location: (6, 2) }), Some(Square { item: None, next: None, location: (6, 3) }), None, None, None, Some(Square { item: None, next: None, location: (6, 7) }), Some(Square { item: None, next: None, location: (6, 8) }), Some(Square { item: None, next: None, location: (6, 9) }), Some(Square { item: None, next: None, location: (6, 10) })], [Some(Square { item: None, next: None, location: (7, 0) }), Some(Square { item: None, next: None, location: (7, 1) }), Some(Square { item: None, next: None, location: (7, 2) }), Some(Square { item: None, next: None, location: (7, 3) }), None, None, None, Some(Square { item: None, next: None, location: (7, 7) }), Some(Square { item: None, next: None, location: (7, 8) }), Some(Square { item: None, next: None, location: (7, 9) }), Some(Square { item: None, next: None, location: (7, 10) })], [None, None, None, Some(Square { item: None, next: None, location: (8, 3) }), Some(Square { item: None, next: None, location: (8, 4) }), Some(Square { item: None, next: None, location: (8, 5) }), Some(Square { item: None, next: None, location: (8, 6) }), Some(Square { item: None, next: None, location: (8, 7) }), None, None, None], [None, None, None, None, None, Some(Square { item: None, next: None, location: (9, 5) }), None, None, None, None, None], [None, None, None, Some(Square { item: None, next: None, location: (10, 3) }), Some(Square { item: None, next: None, location: (10, 4) }), Some(Square { item: None, next: None, location: (10, 5) }), Some(Square { item: None, next: None, location: (10, 6) }), Some(Square { item: None, next: None, location: (10, 7) }), None, None, None], [Some(Square { item: None, next: None, location: (11, 0) }), Some(Square { item: None, next: None, location: (11, 1) }), Some(Square { item: None, next: None, location: (11, 2) }), Some(Square { item: None, next: None, location: (11, 3) }), None, None, None, Some(Square { item: None, next: None, location: (11, 7) }), Some(Square { item: None, next: None, location: (11, 8) }), Some(Square { item: None, next: None, location: (11, 9) }), Some(Square { item: None, next: None, location: (11, 10) })], [Some(Square { item: None, next: None, location: (12, 0) }), Some(Square { item: None, next: None, location: (12, 1) }), Some(Square { item: None, next: None, location: (12, 2) }), Some(Square { item: None, next: None, location: (12, 3) }), None, None, None, Some(Square { item: None, next: None, location: (12, 7) }), Some(Square { item: None, next: None, location: (12, 8) }), Some(Square { item: None, next: None, location: (12, 9) }), Some(Square { item: None, next: None, location: (12, 10) })], [Some(Square { item: None, next: None, location: (13, 0) }), Some(Square { item: None, next: None, location: (13, 1) }), Some(Square { item: None, next: None, location: (13, 2) }), Some(Square { item: None, next: None, location: (13, 3) }), Some(Square { item: None, next: None, location: (13, 4) }), None, Some(Square { item: None, next: None, location: (13, 6) }), Some(Square { item: None, next: None, location: (13, 7) }), Some(Square { item: None, next: None, location: (13, 8) }), Some(Square { item: None, next: None, location: (13, 9) }), Some(Square { item: None, next: None, location: (13, 10) })], [None, Some(Square { item: None, next: None, location: (14, 1) }), Some(Square { item: None, next: None, location: (14, 2) }), Some(Square { item: None, next: None, location: (14, 3) }), Some(Square { item: None, next: None, location: (14, 4) }), Some(Square { item: None, next: None, location: (14, 5) }), Some(Square { item: None, next: None, location: (14, 6) }), Some(Square { item: None, next: None, location: (14, 7) }), Some(Square { item: None, next: None, location: (14, 8) }), Some(Square { item: None, next: None, location: (14, 9) }), None], [None, None, Some(Square { item: None, next: None, location: (15, 2) }), Some(Square { item: None, next: None, location: (15, 3) }), Some(Square { item: None, next: None, location: (15, 4) }), Some(Square { item: None, next: None, location: (15, 5) }), Some(Square { item: None, next: None, location: (15, 6) }), Some(Square { item: None, next: None, location: (15, 7) }), Some(Square { item: None, next: None, location: (15, 8) }), None, None], [None, None, None, Some(Square { item: None, next: None, location: (16, 3) }), Some(Square { item: None, next: None, location: (16, 4) }), Some(Square { item: None, next: None, location: (16, 5) }), Some(Square { item: None, next: None, location: (16, 6) }), Some(Square { item: None, next: None, location: (16, 7) }), None, None, None], [None, None, None, None, Some(Square { item: None, next: None, location: (17, 4) }), Some(Square { item: None, next: None, location: (17, 5) }), Some(Square { item: None, next: None, location: (17, 6) }), None, None, None, None], [None, None, None, None, None, Some(Square { item: None, next: None, location: (18, 5) }), None, None, None, None, None]] }

*/
