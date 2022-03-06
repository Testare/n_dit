use super::{EnemyAi, Pickup, Sprite};
use crate::{Bounds, Direction, GridMap, Point, Team};
use log::debug;
use std::collections::HashSet;

mod with_sprite;

#[derive(Debug)]

//pub struct NodeRestorePoint(GridMap<Piece>);
pub struct NodeRestorePoint();

type NodeConstructionError = String;

#[derive(Clone, Debug)]
pub struct Node {
    grid: GridMap<Piece>,
    name: String,
    active_sprite: Option<usize>,
    enemy_ai: EnemyAi,
    active_team: Team,
}

#[derive(Clone, Debug)]
pub enum Piece {
    AccessPoint,
    Program(Sprite),
    Pickup(Pickup),
}

impl Node {
    // TODO Node undo states
    pub fn create_restore_point(&self) -> NodeRestorePoint {
        NodeRestorePoint()
    }

    fn grid_mut(&mut self) -> &mut GridMap<Piece> {
        &mut self.grid
    }

    fn drop_active_sprite(&mut self) {
        self.active_sprite = None;
    }

    pub fn perform_sprite_action(
        &mut self,
        sprite_action_index: usize,
        target_pt: Point,
    ) -> Option<()> {
        let active_sprite_key = self.active_sprite_key()?;
        let action = self
            .with_sprite(active_sprite_key, |sprite| {
                sprite
                    .actions()
                    .get(sprite_action_index)
                    .map(|action| action.unwrap())
            })
            .flatten()?;
        let result = action.apply(self, active_sprite_key, target_pt);
        match result {
            Ok(()) => {
                self.deactivate_sprite();
                Some(())
            }
            _ => None,
        }
    }

    /// Returns remaining moves
    pub fn move_active_sprite(&mut self, directions: &[Direction]) -> Result<Vec<Pickup>, String> {
        self.with_active_sprite_mut(|mut sprite| sprite.move_sprite(directions))
            .unwrap_or_else(|| Err("No active sprite".to_string()))
    }

    pub fn active_sprite_key(&self) -> Option<usize> {
        self.active_sprite
    }

    pub fn deactivate_sprite(&mut self) {
        self.with_active_sprite_mut(|mut sprite| sprite.tap());
        self.active_sprite = None;
    }

    pub fn activate_sprite(&mut self, sprite_key: usize) -> bool {
        let can_activate = self
            .with_sprite(sprite_key, |sprite| {
                sprite.team() == self.active_team() && !sprite.tapped()
            })
            .unwrap_or(false);

        if can_activate {
            self.with_active_sprite_mut(|mut sprite|
                if sprite.moves_taken() != 0 {
                    debug!("Activating new sprite, old sprite had taken {:?} moves and so must be tapped", sprite.moves_taken());
                    sprite.tap();
                });
            self.active_sprite = Some(sprite_key);
        }
        can_activate
    }

    pub(crate) fn grid(&self) -> &GridMap<Piece> {
        &self.grid
    }

    pub fn add_sprite(
        &mut self,
        spr: Sprite,
        pt_vec: Vec<Point>,
    ) -> Result<usize, NodeConstructionError> {
        // Could possibly be optimized with GridMap::put_entries
        let mut pts = pt_vec.into_iter();
        let first_pt = pts.next().ok_or("Sprite needs at least one point!")?;
        let key = self
            .grid
            .put_item(first_pt, Piece::Program(spr))
            .ok_or_else::<NodeConstructionError, _>(|| {
                "Could not add sprite to initial location".into()
            })?;
        for pt in pts {
            if !self.grid.push_front(pt, key) {
                return Err(format!("Could not add sprite to location {:?}", pt));
            }
        }
        Ok(key)
    }

    pub fn add_piece(&mut self, pt: Point, piece: Piece) -> Option<usize> {
        self.grid.put_item(pt, piece)
    }

    pub fn add_money(&mut self, pt: Point, amount: usize) -> Option<usize> {
        self.grid.put_item(pt, Piece::Pickup(Pickup::Mon(amount)))
    }

    pub fn width(&self) -> usize {
        self.grid.width()
    }

    pub fn height(&self) -> usize {
        self.grid.height()
    }

    pub fn bounds(&self) -> Bounds {
        Bounds::of(self.grid.width(), self.grid.height())
    }

    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    pub fn piece(&self, piece_key: usize) -> Option<&Piece> {
        self.grid.item(piece_key)
    }

    pub fn piece_at(&self, pt: Point) -> Option<&Piece> {
        self.grid.item_at(pt)
    }

    pub fn piece_key_at(&self, pt: Point) -> Option<usize> {
        self.grid.item_key_at(pt)
    }

    // TODO move to WithSprite
    pub fn possible_moves(&self, sprite_key: usize) -> HashSet<Point> {
        let piece = self.grid.item(sprite_key).unwrap();
        let bounds = self.bounds();
        if let Piece::Program(sprite) = piece {
            if sprite.moves() == 0 || sprite.tapped() {
                return HashSet::default();
            }
            fn possible_moves_recur(
                point: Point,
                hash_set: HashSet<Point>,
                moves: usize,
                bounds: &Bounds,
                sprite_key: usize,
                grid: &GridMap<Piece>,
            ) -> HashSet<Point> {
                if moves == 0 {
                    hash_set
                } else {
                    Direction::EVERY_DIRECTION
                        .iter()
                        .fold(hash_set, |mut set, dir| {
                            let next_pt = dir.add_to_point(point, 1, *bounds);
                            if grid.square_is_free(next_pt)
                                || grid.item_key_at(next_pt) == Some(sprite_key)
                            {
                                set.insert(next_pt);
                                possible_moves_recur(
                                    next_pt,
                                    set,
                                    moves - 1,
                                    bounds,
                                    sprite_key,
                                    grid,
                                )
                            } else {
                                set
                            }
                        })
                }
            }
            let head = self.grid.head(sprite_key).unwrap();
            let moves = sprite.moves();
            let mut point_set = HashSet::new();
            point_set.insert(head);
            possible_moves_recur(head, point_set, moves, &bounds, sprite_key, self.grid())
        } else {
            HashSet::default()
        }
    }

    pub fn piece_len(&self, piece_key: usize) -> usize {
        self.grid.len_of(piece_key)
    }

    pub fn active_team(&self) -> Team {
        self.active_team
    }

    pub fn change_active_team(&mut self) {
        let active_team = match self.active_team {
            Team::EnemyTeam => Team::PlayerTeam,
            Team::PlayerTeam => Team::EnemyTeam,
        };
        self.active_team = active_team;
        for sprite_key in self
            .filtered_sprite_keys(|_, sprite| sprite.team() == active_team)
            .iter()
        {
            self.with_sprite_mut(*sprite_key, |mut sprite| sprite.untap());
        }
    }

    pub fn enemy_ai(&self) -> &EnemyAi {
        &self.enemy_ai
    }

    pub fn pieces(&self) -> Vec<&Piece> {
        self.grid().entries()
    }

    pub fn filtered_sprite_keys<P: Fn(usize, &Sprite) -> bool>(&self, predicate: P) -> Vec<usize> {
        self.grid.filtered_keys(|key, piece| {
            if let Piece::Program(sprite) = piece {
                predicate(key, sprite)
            } else {
                false
            }
        })
    }
}

impl Piece {
    pub fn is_pickup(&self) -> bool {
        matches!(self, Piece::Pickup(_))
    }
}

impl From<GridMap<Piece>> for Node {
    fn from(grid: GridMap<Piece>) -> Self {
        Node {
            active_sprite: None,
            active_team: Team::PlayerTeam,
            enemy_ai: EnemyAi::Simple,
            grid,
            name: String::from("Node"),
        }
    }
}

impl From<(String, GridMap<Piece>)> for Node {
    fn from((name, grid): (String, GridMap<Piece>)) -> Self {
        Node {
            active_sprite: None,
            active_team: Team::PlayerTeam,
            enemy_ai: EnemyAi::Simple,
            grid,
            name,
        }
    }
}

impl From<(String, GridMap<Piece>, EnemyAi)> for Node {
    fn from((name, grid, enemy_ai): (String, GridMap<Piece>, EnemyAi)) -> Self {
        Node {
            active_sprite: None,
            active_team: Team::PlayerTeam,
            enemy_ai,
            grid,
            name,
        }
    }
}
