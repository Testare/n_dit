pub type Point = (usize, usize);

// TODO use this 
pub struct Bounds(usize, usize);

pub enum Direction {
    North,
    East,
    South,
    West
}

impl Direction {
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
                    (bounds.width(), point.1)
                } else {
                    (point.0 + speed, point.1)
                }
            }
            Self::South => {
                if point.1 + speed >= bounds.height() {
                    (point.0, bounds.height())
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

}