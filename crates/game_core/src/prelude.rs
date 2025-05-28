pub use std::ops::Deref as _;

pub use bevy::asset::AsyncReadExt;
pub use bevy::platform::collections::{HashMap, HashSet};
pub use bevy::prelude::{
    default, Added, App, AppTypeRegistry, ApplyDeferred as apply_deferred, Asset, AssetApp,
    AssetEvent, AssetServer, Assets, AudioSource, Bundle, Changed, ChildOf, Children, Commands,
    Component, Deref, DerefMut, DetectChanges, DetectChangesMut, DynamicSceneRoot, Entity,
    EntityMapper, Event, EventReader, EventWriter, First, FromReflect, FromWorld, Handle, IVec2,
    In, IntoScheduleConfigs, Last, Local, Mut, Name, NameOrEntity, Or, ParamSet, PlaybackSettings,
    Plugin, PostStartup, PostUpdate, PreStartup, PreUpdate, Query, Ref, Reflect, ReflectComponent,
    ReflectDeserialize, ReflectSerialize, RemovedComponents, Res, ResMut, Resource, SceneRoot,
    Startup, SystemParamFunction, SystemSet, TypePath, UVec2, UVec3, UVec4, Update, Vec2, With,
    Without, World,
};
pub use bevy_query_ext::prelude::*;

pub use crate::common::*;
pub use crate::entity_grid::commands::{AddToGrid, AddToGridCommand};
pub use crate::entity_grid::EntityGrid;
pub use crate::{get_assert, get_assert_mut}; // I use this so much
