use super::Sprite;
use crate::{Bounds, Direction, GridMap, Point, Team};
use std::collections::HashSet;

#[derive(Debug)]
pub struct Node {
    grid: GridMap<Piece>,
    name: String,
    active_sprite: Option<usize>,
}

#[derive(Debug)]
pub enum Piece {
    AccessPoint,
    Program(Sprite),
    Mon(u32),
}

impl Node {
    #[deprecated]
    pub(super) fn grid_mut(&mut self) -> &mut GridMap<Piece> {
        &mut self.grid
    }

    // TODO Idea: Make these functions return a tuple of key and sprite?
    pub fn active_sprite(&self) -> Option<&Sprite> {
        self.active_sprite.and_then(|sprite_key| {
            match self.grid.item(sprite_key) {
                Some(Piece::Program(sprite)) => Some(sprite),
                None => panic!("Somehow the active sprite was deleted without being deactivated"),
                _ => panic!("Somehow a non-sprite was activated")
            }
        })
    }

    pub fn active_sprite_mut(&mut self) -> Option<&mut Sprite> {
        self.active_sprite.map(move |sprite_key| {
            if let Some(Piece::Program(sprite)) = self.grid.item_mut(sprite_key) {
                sprite
            } else {
                panic!("Somehow a non-sprite was activated")
            }
        })
    }

    pub fn active_sprite_key(&self) -> Option<usize> {
        self.active_sprite
    }

    pub fn deactivate_sprite(&mut self) {
        self.active_sprite_mut().map(|sprite| sprite.tap());
        self.active_sprite = None;
    }

    pub fn activate_sprite(&mut self, sprite_key: usize) -> bool {
        let can_activate = self
            .with_sprite(sprite_key, |sprite| {
                sprite.team() == Team::PlayerTeam && !sprite.tapped()
            })
            .unwrap_or(false);
        if can_activate {
            self.active_sprite = Some(sprite_key);
        }
        can_activate
    }

    #[deprecated]
    pub(crate) fn grid(&self) -> &GridMap<Piece> {
        &self.grid
    }

    // TODO sprite builder pattern
    pub fn add_sprite(&mut self, pt: Point, spr: Sprite) -> Option<usize> {
        self.grid.put_item(pt, Piece::Program(spr))
    }

    pub fn add_piece(&mut self, pt: Point, piece: Piece) -> Option<usize> {
        self.grid.put_item(pt, piece)
    }

    pub fn add_money(&mut self, pt: Point, amount: u32) -> Option<usize> {
        self.grid.put_item(pt, Piece::Mon(amount))
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

    // Stubby
    pub fn move_sprite(&mut self, pt: Point, key: usize) -> bool {
        self.grid.push_front(pt, key)
    }

    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    pub fn piece_at(&self, pt: Point) -> Option<&Piece> {
        self.grid.item_at(pt)
    }

    pub fn piece_key_at(&self, pt: Point) -> Option<usize> {
        self.grid.item_key_at(pt)
    }

    pub fn with_sprite_mut<T, R: Into<Option<T>>, F: FnOnce(&mut Sprite) -> R>(
        &mut self,
        sprite_key: usize,
        sprite_op: F,
    ) -> Option<T> {
        if let Some(Piece::Program(sprite)) = self.grid.item_mut(sprite_key) {
            sprite_op(sprite).into()
        } else {
            None
        }
    }

    pub fn with_sprite<T, R: Into<Option<T>>, F: FnOnce(&Sprite) -> R>(
        &self,
        sprite_key: usize,
        sprite_op: F,
    ) -> Option<T> {
        if let Some(Piece::Program(sprite)) = self.grid.item(sprite_key) {
            sprite_op(sprite).into()
        } else {
            None
        }
    }

    pub fn with_sprite_at<T, R: Into<Option<T>>, F: FnOnce(&Sprite) -> R>(
        &self,
        pt: Point,
        sprite_op: F,
    ) -> Option<T> {
        if let Some(Piece::Program(sprite)) = self.grid.item_at(pt) {
            sprite_op(sprite).into()
        } else {
            None
        }
    }

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
}

impl From<GridMap<Piece>> for Node {
    fn from(grid: GridMap<Piece>) -> Self {
        Node {
            active_sprite: None,
            grid,
            name: String::from("Node"),
        }
    }
}

impl From<(String, GridMap<Piece>)> for Node {
    fn from((name, grid): (String, GridMap<Piece>)) -> Self {
        Node {
            active_sprite: None,
            grid,
            name,
        }
    }
}
