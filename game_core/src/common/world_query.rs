use std::borrow::{Borrow, Cow};
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};

use bevy::ecs::archetype::Archetype;
use bevy::ecs::component::ComponentId;
use bevy::ecs::query::{WorldQuery, FilteredAccess, ReadOnlyWorldQuery};
use bevy::ecs::storage::Table;
use bevy::ecs::world::unsafe_world_cell::UnsafeWorldCell;
use bevy::prelude::Component;


pub type AsDerefCopiedOrDefault<T> = OrDefault<Copied<AsDeref<T>>>;
pub type AsDerefClonedOrDefault<T> = OrDefault<Cloned<AsDeref<T>>>;



/// An empty structure type
/// Used to simplify the different modified queries
/// so we don't have as much boilerplate for all the implementations
pub struct ModQ<T>(PhantomData<T>);
pub struct ModQMut<T>(PhantomData<T>);

pub struct CopiedQ<T>(PhantomData<T>);
pub struct ClonedQ<T>(PhantomData<T>);
pub struct AsDerefQ<T>(PhantomData<T>);
pub struct AsDerefMutQ<T>(PhantomData<T>);
pub struct OrDefaultQ<T>(PhantomData<T>);

/// Traits

pub trait ModQuery {
    type FromQuery: ReadOnlyWorldQuery;
    type ModItem<'q>;

    fn modify_reference<'s>(from: <Self::FromQuery as WorldQuery>::Item<'s>) -> Self::ModItem<'s>;

    fn shrink<'wlong: 'wshort, 'wshort>(item: Self::ModItem<'wlong>) -> Self::ModItem<'wshort>;
}

pub trait ModQueryMut {
    type FromQuery: WorldQuery;
    type ModItem<'q>; 
    type ReadOnly: ReadOnlyWorldQuery<State = <<Self as ModQueryMut>::FromQuery as WorldQuery>::State>;

    fn modify_reference<'s>(from: <Self::FromQuery as WorldQuery>::Item<'s>) -> Self::ModItem<'s>;

    fn shrink<'wlong: 'wshort, 'wshort>(item: Self::ModItem<'wlong>) -> Self::ModItem<'wshort>;
}

/// ModifiedQuery: Components

pub type Cloned<T> = ModQ<ClonedQ<T>>;
impl <T: Component + Clone> ModQuery for ClonedQ<T> {
    type FromQuery = &'static T;
    type ModItem<'a> = T;

    fn modify_reference<'s>(t: <Self::FromQuery as WorldQuery>::Item<'s>) -> Self::ModItem<'s> {
        t.clone()
    }

    fn shrink<'wlong: 'wshort, 'wshort>(item: Self::ModItem<'wlong>) -> Self::ModItem<'wshort> {
        item
    }
}

pub type Copied<T> = ModQ<CopiedQ<T>>;
impl <T: Component + Copy> ModQuery for CopiedQ<T> {
    type FromQuery = &'static T;
    type ModItem<'a> = T;

    fn modify_reference<'s>(t: <Self::FromQuery as WorldQuery>::Item<'s>) -> Self::ModItem<'s> {
        *t
    }

    fn shrink<'wlong: 'wshort, 'wshort>(item: Self::ModItem<'wlong>) -> Self::ModItem<'wshort> {
        item
    }
}

pub type AsDeref<T> = ModQ<AsDerefQ<T>>;
impl <T: Component + Deref> ModQuery for AsDerefQ<T> {
    type FromQuery = &'static T;
    type ModItem<'a> = &'a <T as Deref>::Target;

    fn modify_reference<'s>(t: <Self::FromQuery as WorldQuery>::Item<'s>) -> Self::ModItem<'s> {
        t.deref()
    }

    fn shrink<'wlong: 'wshort, 'wshort>(item: Self::ModItem<'wlong>) -> Self::ModItem<'wshort> {
        item
    }
}

// ModQueryMut

pub type AsDerefMut<T> = ModQMut<AsDerefMutQ<T>>;
impl <T: Component + DerefMut> ModQueryMut for AsDerefMutQ<T> {
    type FromQuery = &'static mut T;
    type ModItem<'a> = &'a mut <T as Deref>::Target;
    type ReadOnly = AsDeref<T>;

    fn modify_reference<'s>(t: <Self::FromQuery as WorldQuery>::Item<'s>) -> Self::ModItem<'s> {
        t.into_inner().deref_mut()
    }

    fn shrink<'wlong: 'wshort, 'wshort>(item: Self::ModItem<'wlong>) -> Self::ModItem<'wshort> {
        item
    }
} 


/// ModQuery: Component-level composed

pub type AsDerefCopied<T> = Copied<AsDeref<T>>;
impl <T: Component + Deref> ModQuery for CopiedQ<AsDeref<T>> 
    where <T as Deref>::Target: Copy
{
    type FromQuery = &'static T;
    type ModItem<'a> = <T as Deref>::Target;

    fn modify_reference<'s>(t: <Self::FromQuery as WorldQuery>::Item<'s>) -> Self::ModItem<'s> {
        *t.deref()
    }

    fn shrink<'wlong: 'wshort, 'wshort>(item: Self::ModItem<'wlong>) -> Self::ModItem<'wshort> {
        item
    }
}

pub type AsDerefCloned<T> = Cloned<AsDeref<T>>;
impl <T: Component + Deref> ModQuery for ClonedQ<AsDeref<T>> 
    where <T as Deref>::Target: Clone
{
    type FromQuery = &'static T;
    type ModItem<'a> = <T as Deref>::Target;

    fn modify_reference<'s>(t: <Self::FromQuery as WorldQuery>::Item<'s>) -> Self::ModItem<'s> {
        t.deref().clone()
    }

    fn shrink<'wlong: 'wshort, 'wshort>(item: Self::ModItem<'wlong>) -> Self::ModItem<'wshort> {
        item
    }
}

pub type CopiedAsDeref<T> = AsDeref<Copied<T>>;
impl <T: Component + Copy + Deref> ModQuery for AsDerefQ<Copied<T>> 
{
    type FromQuery = &'static T;
    type ModItem<'a> = &'a <T as Deref>::Target;

    fn modify_reference<'s>(t: <Self::FromQuery as WorldQuery>::Item<'s>) -> Self::ModItem<'s> {
        (*t).deref()
    }

    fn shrink<'wlong: 'wshort, 'wshort>(item: Self::ModItem<'wlong>) -> Self::ModItem<'wshort> {
        item
    }
}

pub type AsDerefCopiedOfCopied<T> = Copied<AsDeref<Copied<T>>>;
impl <T: Component + Copy + Deref> ModQuery for CopiedQ<AsDeref<Copied<T>>> 
    where <T as Deref>::Target: Copy
{
    type FromQuery = &'static T;
    type ModItem<'a> = <T as Deref>::Target;

    fn modify_reference<'s>(t: <Self::FromQuery as WorldQuery>::Item<'s>) -> Self::ModItem<'s> {
        *(*t).deref()
    }

    fn shrink<'wlong: 'wshort, 'wshort>(item: Self::ModItem<'wlong>) -> Self::ModItem<'wshort> {
        item
    }
}

pub type AsDerefCopiedOfCloned<T> = Copied<AsDeref<Cloned<T>>>;
impl <T: Component + Clone + Deref> ModQuery for CopiedQ<AsDeref<Cloned<T>>> 
    where <T as Deref>::Target: Copy
{
    type FromQuery = &'static T;
    type ModItem<'a> = <T as Deref>::Target;

    fn modify_reference<'s>(t: <Self::FromQuery as WorldQuery>::Item<'s>) -> Self::ModItem<'s> {
        *t.clone().deref()
    }

    fn shrink<'wlong: 'wshort, 'wshort>(item: Self::ModItem<'wlong>) -> Self::ModItem<'wshort> {
        item
    }
}

pub type AsDerefClonedOfCloned<T> = Cloned<AsDeref<Cloned<T>>>;
impl <T: Component + Clone + Deref> ModQuery for ClonedQ<AsDeref<Cloned<T>>> 
    where <T as Deref>::Target: Clone
{
    type FromQuery = &'static T;
    type ModItem<'a> = <T as Deref>::Target;

    fn modify_reference<'s>(t: <Self::FromQuery as WorldQuery>::Item<'s>) -> Self::ModItem<'s> {
        t.clone().deref().clone()
    }

    fn shrink<'wlong: 'wshort, 'wshort>(item: Self::ModItem<'wlong>) -> Self::ModItem<'wshort> {
        item
    }
}

pub type AsDerefCopiedOfCopiedOrDefault<T> = Copied<AsDeref<OrDefault<Copied<T>>>>;
impl <T: Component + Copy + Deref + Default> ModQuery for CopiedQ<AsDeref<OrDefault<Copied<T>>>> 
    where <T as Deref>::Target: Copy
{
    type FromQuery = Option<&'static T>;
    type ModItem<'a> = <T as Deref>::Target;

    fn modify_reference<'s>(t: <Self::FromQuery as WorldQuery>::Item<'s>) -> Self::ModItem<'s> {
        *t.copied().unwrap_or_default().deref()
    }

    fn shrink<'wlong: 'wshort, 'wshort>(item: Self::ModItem<'wlong>) -> Self::ModItem<'wshort> {
        item
    }
}


pub type AsDerefCopiedOfClonedOrDefault<T> = Copied<AsDeref<OrDefault<Cloned<T>>>>;
impl <T: Component + Clone + Deref + Default> ModQuery for CopiedQ<AsDeref<OrDefault<Cloned<T>>>> 
    where <T as Deref>::Target: Copy
{
    type FromQuery = Option<&'static T>;
    type ModItem<'a> = <T as Deref>::Target;

    fn modify_reference<'s>(t: <Self::FromQuery as WorldQuery>::Item<'s>) -> Self::ModItem<'s> {
        *t.cloned().unwrap_or_default().deref()
    }

    fn shrink<'wlong: 'wshort, 'wshort>(item: Self::ModItem<'wlong>) -> Self::ModItem<'wshort> {
        item
    }
}

pub type AsDerefClonedOfClonedOrDefault<T> = Cloned<AsDeref<OrDefault<Cloned<T>>>>;
impl <T: Component + Clone + Deref + Default> ModQuery for ClonedQ<AsDeref<OrDefault<Cloned<T>>>> 
    where <T as Deref>::Target: Clone
{
    type FromQuery = Option<&'static T>;
    type ModItem<'a> = <T as Deref>::Target;

    fn modify_reference<'s>(t: <Self::FromQuery as WorldQuery>::Item<'s>) -> Self::ModItem<'s> {
        t.cloned().unwrap_or_default().deref().clone()
    }

    fn shrink<'wlong: 'wshort, 'wshort>(item: Self::ModItem<'wlong>) -> Self::ModItem<'wshort> {
        item
    }
}

// ModQuery: OrX, works on any readonly query

pub type OrDefault<T> = ModQ<OrDefaultQ<T>>;
pub type CopiedOrDefault<T> = OrDefault<Copied<T>>; 
pub type ClonedOrDefault<T> = OrDefault<Cloned<T>>;

impl <T: ReadOnlyWorldQuery> ModQuery for OrDefaultQ<T> 
    where for <'a> <T as WorldQuery>::Item<'a>: Default {
    type FromQuery = Option<T>;
    type ModItem<'s> = T::Item<'s>;

    fn modify_reference<'s>(t: <Self::FromQuery as WorldQuery>::Item<'s>) -> Self::ModItem<'s> {
        t.unwrap_or_default()
    }

    fn shrink<'wlong: 'wshort, 'wshort>(item: Self::ModItem<'wlong>) -> Self::ModItem<'wshort> {
        <T as WorldQuery>::shrink(item)
    }
}

macro_rules! or_const {
    ($OrConst:ident, $OrConstQ:ident, $const_type:ty) => {
        pub struct $OrConstQ<const V: $const_type, T>(PhantomData<T>);

        pub type $OrConst<const V: $const_type, T> = ModQ<$OrConstQ<V, T>>;

        impl <T: ReadOnlyWorldQuery, const V: $const_type> ModQuery for $OrConstQ<V, T> 
            where for<'a> <T as WorldQuery>::Item<'a>: Borrow<$const_type> {
            type FromQuery = Option<T>;
            type ModItem<'s> = $const_type;

            fn modify_reference<'s>(t: <Self::FromQuery as WorldQuery>::Item<'s>) -> Self::ModItem<'s> {
                t.map(|b|*b.borrow()).unwrap_or(V)
            }

            fn shrink<'wlong: 'wshort, 'wshort>(item: Self::ModItem<'wlong>) -> Self::ModItem<'wshort> {
                item
            }
        }
    }
}

or_const!(OrBool, OrBoolQ, bool);
or_const!(OrIsize, OrIsizeQ, isize);
or_const!(OrUsize, OrUsizeQ, usize);
or_const!(OrI32, OrI32Q, i32);
or_const!(OrU32, OrU32Q, u32);
or_const!(OrI16, OrI16Q, i16);
or_const!(OrU16, OrU16Q, u16);
or_const!(OrI8, OrI8Q, i8);
or_const!(OrU8, OrU8Q, u8);

// Implementing WorldQuery/ReadOnlyWorldQuery

unsafe impl <T: ModQuery> WorldQuery for ModQ<T> {
    type Fetch<'w> = <T::FromQuery as WorldQuery>::Fetch<'w>;
    type Item<'w> = T::ModItem<'w>;
    type ReadOnly = Self;
    type State = <T::FromQuery as WorldQuery>::State;

    fn shrink<'wlong: 'wshort, 'wshort>(item: Self::Item<'wlong>) -> Self::Item<'wshort> {
        T::shrink(item)
    }

    const IS_DENSE: bool = <T::FromQuery>::IS_DENSE;
    const IS_ARCHETYPAL: bool = <T::FromQuery>::IS_ARCHETYPAL;

    #[inline]
    unsafe fn init_fetch<'w>(
        world: UnsafeWorldCell<'w>,
        state: &Self::State,
        last_run: bevy::ecs::component::Tick,
        this_run: bevy::ecs::component::Tick
    ) -> Self::Fetch<'w> {
        <T::FromQuery as WorldQuery>::init_fetch(world, state, last_run, this_run)
    }

    #[inline]
    unsafe fn set_archetype<'w>(
        fetch: &mut Self::Fetch<'w>,
        state: &Self::State,
        archetype: &'w Archetype,
        table: &'w Table,
    ) {
        <T::FromQuery as WorldQuery>::set_archetype(fetch, state, archetype, table);
    }

    unsafe fn clone_fetch<'w>(fetch: &Self::Fetch<'w>) -> Self::Fetch<'w> {
        <T::FromQuery as WorldQuery>::clone_fetch(fetch)
    }

    unsafe fn set_table<'w>(fetch: &mut Self::Fetch<'w>, state: &Self::State, table: &'w Table) {
        <T::FromQuery as WorldQuery>::set_table(fetch, state, table);
    }

    unsafe fn fetch<'w>(
        fetch: &mut Self::Fetch<'w>,
        entity: bevy::prelude::Entity,
        table_row: bevy::ecs::storage::TableRow,
    ) -> Self::Item<'w> {
        T::modify_reference(<T::FromQuery as WorldQuery>::fetch(fetch, entity, table_row))
    }

    fn update_component_access(state: &Self::State, access: &mut FilteredAccess<ComponentId>) {
        <T::FromQuery as WorldQuery>::update_component_access(state, access)
    }

    fn update_archetype_component_access(
        state: &Self::State,
        archetype: &Archetype,
        access: &mut bevy::ecs::query::Access<bevy::ecs::archetype::ArchetypeComponentId>,
    ) {
        <T::FromQuery as WorldQuery>::update_archetype_component_access(state, archetype, access)
    }

    fn init_state(world: &mut bevy::prelude::World) -> Self::State {
        <T::FromQuery as WorldQuery>::init_state(world)
    }

    fn matches_component_set(
        state: &Self::State,
        set_contains_id: &impl Fn(bevy::ecs::component::ComponentId) -> bool,
    ) -> bool {
        <T::FromQuery as WorldQuery>::matches_component_set(state, set_contains_id)
    }

}

// SAFETY: ModifiedQuery comes from a read only place
unsafe impl <T: ModQuery> ReadOnlyWorldQuery for ModQ<T> {}

unsafe impl <T: ModQueryMut> WorldQuery for ModQMut<T> {
    type Fetch<'w> = <T::FromQuery as WorldQuery>::Fetch<'w>;
    type Item<'w> = T::ModItem<'w>;
    type ReadOnly = T::ReadOnly;
    type State = <T::FromQuery as WorldQuery>::State;

    fn shrink<'wlong: 'wshort, 'wshort>(item: Self::Item<'wlong>) -> Self::Item<'wshort> {
        T::shrink(item)
    }

    const IS_DENSE: bool = <T::FromQuery>::IS_DENSE;
    const IS_ARCHETYPAL: bool = <T::FromQuery>::IS_ARCHETYPAL;

    #[inline]
    unsafe fn init_fetch<'w>(
        world: UnsafeWorldCell<'w>,
        state: &Self::State,
        last_run: bevy::ecs::component::Tick,
        this_run: bevy::ecs::component::Tick
    ) -> Self::Fetch<'w> {
        <T::FromQuery as WorldQuery>::init_fetch(world, state, last_run, this_run)
    }

    #[inline]
    unsafe fn set_archetype<'w>(
        fetch: &mut Self::Fetch<'w>,
        state: &Self::State,
        archetype: &'w Archetype,
        table: &'w Table,
    ) {
        <T::FromQuery as WorldQuery>::set_archetype(fetch, state, archetype, table);
    }

    unsafe fn clone_fetch<'w>(fetch: &Self::Fetch<'w>) -> Self::Fetch<'w> {
        <T::FromQuery as WorldQuery>::clone_fetch(fetch)
    }

    unsafe fn set_table<'w>(fetch: &mut Self::Fetch<'w>, state: &Self::State, table: &'w Table) {
        <T::FromQuery as WorldQuery>::set_table(fetch, state, table);
    }

    unsafe fn fetch<'w>(
        fetch: &mut Self::Fetch<'w>,
        entity: bevy::prelude::Entity,
        table_row: bevy::ecs::storage::TableRow,
    ) -> Self::Item<'w> {
        T::modify_reference(<T::FromQuery as WorldQuery>::fetch(fetch, entity, table_row))
    }

    fn update_component_access(state: &Self::State, access: &mut FilteredAccess<ComponentId>) {
        <T::FromQuery as WorldQuery>::update_component_access(state, access)
    }

    fn update_archetype_component_access(
        state: &Self::State,
        archetype: &Archetype,
        access: &mut bevy::ecs::query::Access<bevy::ecs::archetype::ArchetypeComponentId>,
    ) {
        <T::FromQuery as WorldQuery>::update_archetype_component_access(state, archetype, access)
    }

    fn init_state(world: &mut bevy::prelude::World) -> Self::State {
        <T::FromQuery as WorldQuery>::init_state(world)
    }

    fn matches_component_set(
        state: &Self::State,
        set_contains_id: &impl Fn(bevy::ecs::component::ComponentId) -> bool,
    ) -> bool {
        <T::FromQuery as WorldQuery>::matches_component_set(state, set_contains_id)
    }

}