use std::cmp::{max, min};
use std::collections::HashSet;
use std::convert::TryInto;

pub type Point = (usize, usize);

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
            }
        }
    }

    pub fn as_set(self) -> HashSet<Point> {
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
            }
        }
    }
}

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

#[cfg(test)]
mod test {
    use crate::{Bounds, PointSet};

    #[test]
    #[ignore] // TODO NOT DONE
    pub fn range_of_pt_test() {
        let bounds = Bounds::of(10, 10);
        let pt = (1, 1);
        let set = PointSet::range_of_pt(pt, 1, bounds);
    }
}
