pub type Point = (usize, usize);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Bounds(usize, usize);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Direction {
    North,
    East,
    South,
    West,
}

impl Direction {
    pub const EVERY_DIRECTION: [Direction; 4] = [
        Direction::North,
        Direction::East,
        Direction::South,
        Direction::West,
    ];

    pub fn add_to_point(&self, point: Point, speed: usize, bounds: Bounds) -> Point {
        match self {
            Self::North => {
                if speed >= point.1 {
                    (point.0, 0)
                } else {
                    (point.0, point.1 - speed)
                }
            }
            Self::East => {
                if point.0 + speed >= bounds.width() {
                    (bounds.width() - 1, point.1)
                } else {
                    (point.0 + speed, point.1)
                }
            }
            Self::South => {
                if point.1 + speed >= bounds.height() {
                    (point.0, bounds.height() - 1)
                } else {
                    (point.0, point.1 + speed)
                }
            }
            Self::West => {
                if speed >= point.0 {
                    (0, point.1)
                } else {
                    (point.0 - speed, point.1)
                }
            }
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
