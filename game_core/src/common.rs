use bevy::prelude::UVec2;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Compass {
    North = 1,
    East = 2,
    South = 4,
    West = 8,
}

impl Compass {
    pub const ALL_DIRECTIONS: [Compass; 4] =
        [Compass::North, Compass::East, Compass::South, Compass::West];

    pub fn is_vertical(&self) -> bool {
        match self {
            Compass::North | Compass::South => true,
            Compass::East | Compass::West => false,
        }
    }
}

impl std::ops::Add<Compass> for UVec2 {
    type Output = UVec2;
    fn add(self, rhs: Compass) -> Self::Output {
        let UVec2 { x, y } = self;
        match rhs {
            Compass::North => UVec2 {
                x,
                y: y.saturating_sub(1),
            },
            Compass::East => UVec2 { x: x + 1, y },
            Compass::South => UVec2 { x, y: y + 1 },
            Compass::West => UVec2 {
                x: x.saturating_sub(1),
                y,
            },
        }
    }
}

pub trait GridPoints {
    fn manhattan_distance(&self, rhs: &Self) -> u32;

    fn dirs_to(&self, rhs: &Self) -> [Option<Compass>; 2];
}

impl GridPoints for UVec2 {
    fn manhattan_distance(&self, rhs: &UVec2) -> u32 {
        self.x.abs_diff(rhs.x) + self.y.abs_diff(rhs.y)
    }

    fn dirs_to(&self, rhs: &Self) -> [Option<Compass>; 2] {
        let mut dirs: Vec<Compass> = Vec::new();
        if self.y > rhs.y {
            dirs.push(Compass::North);
        }
        if self.x < rhs.x {
            dirs.push(Compass::East);
        }
        if self.y < rhs.y {
            dirs.push(Compass::South);
        }
        if self.x > rhs.x {
            dirs.push(Compass::West);
        }
        [dirs.get(0).copied(), dirs.get(1).copied()]
    }
}
