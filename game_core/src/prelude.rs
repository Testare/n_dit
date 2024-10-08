pub use std::ops::Deref as _;

pub use bevy::asset::AsyncReadExt;
pub use bevy::prelude::{
    apply_deferred, default, Added, App, AppTypeRegistry, Asset, AssetApp, AssetEvent, AssetServer,
    Assets, AudioBundle, AudioSource, AudioSourceBundle, BuildChildren, Bundle, Changed, Children,
    Commands, Component, DebugName, Deref, DerefMut, DetectChanges, DetectChangesMut, Entity,
    EntityMapper, Event, EventReader, EventWriter, First, FromReflect, FromWorld, Handle,
    HierarchyQueryExt, IVec2, In, IntoSystemConfigs, IntoSystemSetConfigs, Last, Local, Mut, Name,
    Or, ParamSet, Parent, PlaybackSettings, Plugin, PostStartup, PostUpdate, PreStartup, PreUpdate,
    PushChild, Query, Ref, Reflect, ReflectComponent, ReflectDeserialize, ReflectSerialize,
    RemovedComponents, Res, ResMut, Resource, Startup, SystemParamFunction, SystemSet, TypePath,
    UVec2, UVec3, UVec4, Update, Vec2, With, Without, World,
};
pub use bevy::utils::{HashMap, HashSet};
pub use bevy_query_ext::prelude::*;

pub use crate::common::*;
pub use crate::entity_grid::commands::{AddToGrid, AddToGridCommand};
pub use crate::entity_grid::EntityGrid;
pub use crate::{get_assert, get_assert_mut}; // I use this so much
