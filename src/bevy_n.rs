/// Abbreviated form of the bevy prelude to avoid namespace collisions
pub mod prelude {
    pub use bevy::prelude::{Component, Reflect, FromReflect, Plugin, App, Commands, Query, Res, ResMut};
}

pub mod demo;
pub mod game_core;
pub mod term;
