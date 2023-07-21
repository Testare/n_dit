mod grid_map;

use std::cmp::min;
use std::collections::HashSet;
use std::ops::{Add, BitAnd, BitOr};

pub use grid_map::GridMap;
use serde::{Deserialize, Serialize};

pub type Point = (usize, usize);
pub type Pt<T> = (T, T);

#[derive(Clone, Debug)]
pub enum PointSet {
    Range(Point, usize, Bounds),
    Pts(HashSet<Point>),
}

impl PointSet {
    pub fn range_of_pt(pt: Point, range: usize, bounds: Bounds) -> Self {
        PointSet::Range(pt, range, bounds)
    }

    pub fn contains(&self, pt: Point) -> bool {
        match self {
            PointSet::Pts(pts) => pts.contains(&pt),
            PointSet::Range((x, y), range, bounds) => {
                if !bounds.contains_pt(pt) {
                    return false;
                }
                let x_diff = x.checked_sub(pt.0).unwrap_or_else(|| pt.0 - x);
                let y_diff = y.checked_sub(pt.1).unwrap_or_else(|| pt.1 - y);
                let manhattan_distance = x_diff + y_diff;
                manhattan_distance <= *range
            },
        }
    }

    pub fn into_set(self) -> HashSet<Point> {
        match self {
            PointSet::Pts(pts) => pts,
            PointSet::Range((center_x, center_y), range, bounds) => {
                let mut set = HashSet::new();
                let irange = range.try_into().unwrap_or(<isize>::MAX);
                let ix: isize = center_x.try_into().unwrap();
                let iy: isize = center_y.try_into().unwrap();

                let min_y_diff = -min(iy, irange);
                let max_y_diff = min(bounds.height() - center_y, range).try_into().unwrap();
                for y_diff in min_y_diff..=max_y_diff {
                    let range_remaining = irange - y_diff.abs();
                    let min_x_diff = -min(ix, range_remaining);
                    let max_x_diff = min(
                        (bounds.width() - center_x).try_into().unwrap(),
                        range_remaining,
                    );
                    for x_diff in min_x_diff..=max_x_diff {
                        set.insert((
                            (ix + x_diff).try_into().unwrap(),
                            (iy + y_diff).try_into().unwrap(),
                        ));
                    }
                }
                set
            },
        }
    }

    pub fn merge(point_sets: Vec<PointSet>) -> PointSet {
        PointSet::Pts(
            point_sets
                .into_iter()
                .fold(HashSet::<Point>::new(), |acm, point_set| {
                    &acm | &point_set.into_set()
                }),
        )
    }
}

impl BitAnd<PointSet> for PointSet {
    type Output = PointSet;
    fn bitand(self, rhs: PointSet) -> PointSet {
        PointSet::Pts(&self.into_set() & &rhs.into_set())
    }
}

impl Default for PointSet {
    fn default() -> Self {
        PointSet::Pts(HashSet::default())
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Bounds(pub usize, pub usize);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Direction {
    North = 0b1,
    East = 0b10,
    South = 0b100,
    West = 0b1000,
}

impl BitOr for Direction {
    type Output = u8;
    fn bitor(self, rhs: Direction) -> Self::Output {
        self as u8 | rhs as u8
    }
}

impl BitOr<Direction> for u8 {
    type Output = u8;
    fn bitor(self, rhs: Direction) -> Self::Output {
        self | (rhs as u8)
    }
}

impl Direction {
    pub const EVERY_DIRECTION: [Direction; 4] = [
        Direction::North,
        Direction::East,
        Direction::South,
        Direction::West,
    ];

    pub fn vertical(&self) -> bool {
        matches!(self, Direction::North | Direction::South)
    }

    pub fn horizontal(&self) -> bool {
        matches!(self, Direction::East | Direction::West)
    }

    pub fn clockwise(&self) -> Direction {
        match self {
            Direction::North => Direction::East,
            Direction::East => Direction::South,
            Direction::South => Direction::West,
            Direction::West => Direction::South,
        }
    }

    pub fn flip(&self) -> Direction {
        match self {
            Direction::North => Direction::South,
            Direction::East => Direction::West,
            Direction::South => Direction::North,
            Direction::West => Direction::East,
        }
    }

    pub fn matches(&self, directions: u8) -> bool {
        ((*self as u8) & directions) != 0
    }

    pub fn add_to_point(&self, point: Point, speed: usize, bounds: Bounds) -> Point {
        match self {
            Self::North => {
                if speed >= point.1 {
                    (point.0, 0)
                } else {
                    (point.0, point.1 - speed)
                }
            },
            Self::East => {
                if point.0 + speed >= bounds.width() {
                    (bounds.width() - 1, point.1)
                } else {
                    (point.0 + speed, point.1)
                }
            },
            Self::South => {
                if point.1 + speed >= bounds.height() {
                    (point.0, bounds.height() - 1)
                } else {
                    (point.0, point.1 + speed)
                }
            },
            Self::West => {
                if speed >= point.0 {
                    (0, point.1)
                } else {
                    (point.0 - speed, point.1)
                }
            },
        }
    }
}

impl Add<Direction> for Point {
    type Output = Option<Point>;

    fn add(self: Point, rhs: Direction) -> Self::Output {
        match rhs {
            Direction::North => self.1.checked_sub(1).map(|y| (self.0, y)),
            Direction::East => Some((self.0 + 1, self.1)),
            Direction::South => Some((self.0, self.1 + 1)),
            Direction::West => self.0.checked_sub(1).map(|x| (x, self.1)),
        }
    }
}

impl Bounds {
    pub fn of(width: usize, height: usize) -> Self {
        Bounds(width, height)
    }

    pub fn width(&self) -> usize {
        self.0
    }

    pub fn height(&self) -> usize {
        self.1
    }

    pub fn contains_pt(&self, pt: Point) -> bool {
        pt.0 < self.0 && pt.1 < self.1
    }
}

impl From<(u16, u16)> for Bounds {
    fn from((width, height): (u16, u16)) -> Self {
        Bounds(<usize>::from(width), <usize>::from(height))
    }
}

impl From<(usize, usize)> for Bounds {
    fn from((width, height): (usize, usize)) -> Self {
        Bounds(width, height)
    }
}

impl From<Bounds> for (usize, usize) {
    fn from(Bounds(width, height): Bounds) -> Self {
        (width, height)
    }
}
