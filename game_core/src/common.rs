pub mod metadata;

use std::ops::Deref;

use bevy::ecs::query::{QueryEntityError, ReadOnlyWorldQuery, WorldQuery};
use bevy::ecs::system::SystemParam;
use bevy::prelude::{Component, Deref, Entity, Name, ParamSet, Query, Reflect, UVec2, With};
pub use metadata::Metadata;

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

/// Future improvement: iter methods for when there are multiple results
#[derive(SystemParam)]
pub struct IndexedQuery<'w, 's, I, Q, F = ()>(
    Query<'w, 's, (&'static I, Entity)>,
    Query<'w, 's, Q, F>,
)
where
    I: Deref<Target = Entity> + Component,
    Q: WorldQuery + 'static,
    F: ReadOnlyWorldQuery + 'static;

impl<'w, 's, I, Q, F> IndexedQuery<'w, 's, I, Q, F>
where
    I: Deref<Target = Entity> + Component,
    Q: WorldQuery + 'static,
    F: ReadOnlyWorldQuery + 'static,
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

    pub fn get_for(
        &self,
        index: Entity,
    ) -> Result<<<Q as WorldQuery>::ReadOnly as WorldQuery>::Item<'_>, QueryEntityError> {
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
