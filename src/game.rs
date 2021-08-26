use super::{Bounds, GridMap, Point};
use super::ui::UiState;


pub struct SuperState {
    pub ui: UiState,
    pub game: GameState,
}

pub struct GameState {
    world_map: WorldMap,
    node: Option<Node>,
}

// TODO implement
pub struct WorldMap {
    nodes: usize 
}

pub struct Node {
    grid: GridMap<Piece>,
    name: String,
}

// Represent things in the field
// Perhaps we change from enum to struct
pub enum Piece {
    AccessPoint,
    Program(Sprite),
    Mon(u32),
}

#[derive(PartialEq, Eq)]
pub struct Sprite {
    display: String,
    name: String,
    max_size: usize,
    moved: bool,
    team: usize,
    // actions
}

impl Sprite {

    pub fn new(display: &str) -> Sprite {
        Sprite {
            display: String::from(display),
            name: String::from("George"),
            max_size: 3,
            moved: false,
            team: 0
        }
    }
    pub fn display(&self) -> &str {
        self.display.as_ref()
    }

    pub fn name(&self) -> &str {
        self.name.as_ref()
    }
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
        Node { name: String::from("Node"), grid}
    }
}

impl From<(String, GridMap<Piece>)> for Node {
    fn from((name, grid): (String, GridMap<Piece>)) -> Self {
        Node { name, grid}
    }
}

impl GameState {

    pub fn node(&self) -> Option<&Node> {
        self.node.as_ref()
    }

    pub fn from(node: Option<Node>) -> Self {
        GameState {
            node,
            world_map: WorldMap {
                nodes: 1
            }
        }
    }

}

impl SuperState {

    pub fn from(node: Option<Node>) -> Self {
        SuperState {
            game: GameState::from(node),
            ui: UiState::default() 
        }
    }
}