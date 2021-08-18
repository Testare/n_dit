use super::{GridMap, Point, configuration::DrawConfiguration};


pub struct SuperState {
    terminal_size: (usize, usize),
    game_state: GameState,
    draw_config: DrawConfiguration
}

pub struct GameState {
    world_map: WorldMap,
    node: Option<Node>,
}

// UNIMPLEMENTED
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
    max_size: usize,
    moved: bool,
    team: usize,
    // actions
}

impl Sprite {

    pub fn new(display: &str,) -> Sprite {
        Sprite {
            display: String::from(display),
            max_size: 3,
            moved: false,
            team: 0
        }
    }
    pub fn display(&self) -> &str {
        self.display.as_ref()
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

    pub fn add_money(&mut self, pt: Point, amount: u32) -> Option<usize> {
        self.grid.put_item(pt, Piece::Mon(amount))
    }

    // Stubby
    pub fn move_sprite(&mut self, pt: Point, key: usize) -> bool {
        self.grid.push_front(pt, key)
    }

}

impl From<GridMap<Piece>> for Node {
    fn from(grid: GridMap<Piece>) -> Self {
        Node { name: String::from("Node"), grid }
    }
}

impl From<(String, GridMap<Piece>)> for Node {
    fn from((name, grid): (String, GridMap<Piece>)) -> Self {
        Node { name, grid }
    }
}
