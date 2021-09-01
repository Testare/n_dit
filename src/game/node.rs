use super::Sprite;
use crate::{Bounds, Direction, GridMap, Point};
use std::collections::HashSet;

#[derive(Debug)]
pub struct Node {
    grid: GridMap<Piece>,
    name: String,
    activated_sprite: Option<usize>,
}

#[derive(Debug)]
pub enum Piece {
    AccessPoint,
    Program(Sprite),
    Mon(u32),
}

impl Node {
    pub fn activate_sprite(&mut self, sprite_key: usize) -> bool {
        if self.grid.contains_key(sprite_key) {
            true
        } else {
            false
        }
    }

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

    pub fn possible_moves(&self, sprite_key: usize) -> HashSet<Point> {
        let piece = self.grid.item(sprite_key).unwrap();
        let bounds = self.bounds();
        if let Piece::Program(sprite) = piece {
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
}

impl From<GridMap<Piece>> for Node {
    fn from(grid: GridMap<Piece>) -> Self {
        Node {
            name: String::from("Node"),
            activated_sprite: None,
            grid,
        }
    }
}

impl From<(String, GridMap<Piece>)> for Node {
    fn from((name, grid): (String, GridMap<Piece>)) -> Self {
        Node {
            activated_sprite: None,
            name,
            grid,
        }
    }
}
