pub mod daddy;
pub mod metadata;
pub mod sord;

use std::ops::Deref;
use std::str::FromStr;

use bevy::ecs::query::{QueryData, QueryEntityError, QueryFilter, WorldQuery};
use bevy::ecs::system::SystemParam;
use bevy::prelude::{Component, Entity, Query, Reflect, UVec2};
use bevy::reflect::{ReflectDeserialize, ReflectSerialize};
pub use metadata::Metadata;
use serde::{Deserialize, Serialize};

/// Unique identifier for a member of a set of 32 things or less.
/// See [crate::node::NodeId], which will probably be replaced with this later
#[derive(Clone, Debug, Default, Deserialize, Hash, PartialEq, Reflect, Serialize)]
#[reflect(Deserialize, Serialize)]
pub struct SetId {
    /// Nodes are within sets of up to 32 nodes
    set: String,
    /// Number of node in series,
    num: u32,
}

#[derive(Debug)]
pub enum SetIdError {
    NoColon,
    NumTooHigh,
    NumNotANumber(<u32 as FromStr>::Err),
}

impl FromStr for SetId {
    type Err = SetIdError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut split = s.split(':');
        let set = split
            .next()
            .expect("split should always return at least one item")
            .to_string();
        let num_unparsed = split.next().ok_or(SetIdError::NoColon)?;
        let num = num_unparsed.parse().map_err(SetIdError::NumNotANumber)?;
        Ok(Self { set, num })
    }
}

impl std::fmt::Display for SetId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.set.as_str(), self.num)
    }
}

impl SetId {
    /// ## Panics
    /// Panics if num is 32 or more
    pub fn new_unchecked<S: ToString>(set: S, num: u32) -> Self {
        debug_assert!(num < 32, "Node ID has invalid value: [{}] >= 32", num);
        Self {
            num,
            set: set.to_string(),
        }
    }

    pub fn new<S: ToString>(set: S, num: u32) -> Option<Self> {
        if num < 32 {
            Some(Self {
                num,
                set: set.to_string(),
            })
        } else {
            None
        }
    }

    pub fn num(&self) -> u32 {
        self.num
    }

    pub fn set(&self) -> &str {
        self.set.as_str()
    }

    pub fn num_flag(&self) -> u32 {
        1 << self.num
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Reflect)]
pub enum Compass {
    North = 1,
    East = 2,
    South = 4,
    West = 8,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Reflect)]
pub enum CompassOrPoint {
    Compass(Compass),
    Point(UVec2),
}

impl CompassOrPoint {
    pub fn point_from(&self, from: UVec2) -> UVec2 {
        match self {
            Self::Compass(compass) => from + *compass,
            Self::Point(point) => *point,
        }
    }
}

impl From<UVec2> for CompassOrPoint {
    fn from(value: UVec2) -> Self {
        CompassOrPoint::Point(value)
    }
}

impl From<Compass> for CompassOrPoint {
    fn from(value: Compass) -> Self {
        CompassOrPoint::Compass(value)
    }
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

    pub fn is_horizontal(&self) -> bool {
        match self {
            Compass::North | Compass::South => false,
            Compass::East | Compass::West => true,
        }
    }
}

impl std::ops::Neg for Compass {
    type Output = Self;
    fn neg(self) -> Self::Output {
        match self {
            Self::North => Self::South,
            Self::East => Self::West,
            Self::South => Self::North,
            Self::West => Self::East,
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

impl std::ops::Sub<Compass> for UVec2 {
    type Output = UVec2;
    fn sub(self, rhs: Compass) -> Self::Output {
        let UVec2 { x, y } = self;
        match rhs {
            Compass::North => UVec2 { x, y: y + 1 },
            Compass::East => UVec2 {
                x: x.saturating_sub(1),
                y,
            },
            Compass::South => UVec2 {
                x,
                y: y.saturating_sub(1),
            },
            Compass::West => UVec2 { x: x + 1, y },
        }
    }
}

impl std::fmt::Display for Compass {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::North => "north",
                Self::East => "east",
                Self::South => "south",
                Self::West => "west",
            }
        )
    }
}

pub trait GridPoints {
    fn manhattan_distance(&self, rhs: &Self) -> u32;

    fn dirs_to(&self, rhs: &Self) -> [Option<Compass>; 2];

    fn dist_to_pt_along_compass(&self, rhs: &Self, dir: Compass) -> i32;
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
        [dirs.first().copied(), dirs.get(1).copied()]
    }

    fn dist_to_pt_along_compass(&self, rhs: &Self, dir: Compass) -> i32 {
        match dir {
            Compass::North => self.y as i32 - rhs.y as i32,
            Compass::East => rhs.x as i32 - self.x as i32,
            Compass::South => rhs.y as i32 - self.y as i32,
            Compass::West => self.x as i32 - rhs.x as i32,
        }
    }
}

/// Future improvement: iter methods for when there are multiple results
#[derive(Debug, SystemParam)]
pub struct IndexedQuery<'w, 's, I, Q, F = ()>(
    Query<'w, 's, (&'static I, Entity)>,
    Query<'w, 's, Q, F>,
)
where
    I: Deref<Target = Entity> + Component,
    Q: QueryData + 'static,
    F: QueryFilter + 'static;

impl<'w, 's, I, Q, F> IndexedQuery<'w, 's, I, Q, F>
where
    I: Deref<Target = Entity> + Component,
    Q: QueryData + 'static,
    F: QueryFilter + 'static,
{
    pub fn unindexed(&self) -> &Query<'w, 's, Q, F> {
        &self.1
    }

    pub fn unindex_mut(&mut self) -> &mut Query<'w, 's, Q, F> {
        &mut self.1
    }

    pub fn into_unindexed(self) -> Query<'w, 's, Q, F> {
        self.1
    }

    /// If there are multiple, returns the first one it finds
    pub fn id_for(&self, index: Entity) -> Option<Entity> {
        self.0.iter().find_map(|(i, id)| {
            if **i == index && self.1.contains(id) {
                Some(id)
            } else {
                None
            }
        })
    }

    // Repalce name with "one_to_one"
    pub fn get_for(
        &self,
        index: Entity,
    ) -> Result<<<Q as QueryData>::ReadOnly as WorldQuery>::Item<'_>, QueryEntityError> {
        if let Some(id) = self.id_for(index) {
            self.1.get(id)
        } else {
            Err(QueryEntityError::NoSuchEntity(index))
        }
    }

    pub fn get_for_mut(
        &mut self,
        index: Entity,
    ) -> Result<<Q as WorldQuery>::Item<'_>, QueryEntityError> {
        if let Some(id) = self.id_for(index) {
            self.1.get_mut(id)
        } else {
            Err(QueryEntityError::NoSuchEntity(index))
        }
    }
}

#[cfg(test)]
pub mod test {
    use bevy::math::UVec2;

    use crate::Compass;

    #[test]
    fn point_minus_compass_equals_points_plus_minus_compass() {
        let pt = UVec2 { x: 3, y: 3 };
        for dir in Compass::ALL_DIRECTIONS {
            assert_eq!(
                pt - dir,
                pt + -dir,
                "{pt:?} - {dir:?} should be the same as adding the inverse"
            );
        }
    }
}
