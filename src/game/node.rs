use super::super::{Bounds, GridMap, Point};
use super::Sprite;

pub struct Node {
    grid: GridMap<Piece>,
    name: String,
}

pub enum Piece {
    AccessPoint,
    Program(Sprite),
    Mon(u32),
}

impl Node {
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
}

impl From<GridMap<Piece>> for Node {
    fn from(grid: GridMap<Piece>) -> Self {
        Node {
            name: String::from("Node"),
            grid,
        }
    }
}

impl From<(String, GridMap<Piece>)> for Node {
    fn from((name, grid): (String, GridMap<Piece>)) -> Self {
        Node { name, grid }
    }
}
