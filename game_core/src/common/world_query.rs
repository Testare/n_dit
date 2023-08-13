use std::borrow::Borrow;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};

use bevy::ecs::archetype::Archetype;
use bevy::ecs::component::ComponentId;
use bevy::ecs::query::{WorldQuery, FilteredAccess, ReadOnlyWorldQuery};
use bevy::ecs::storage::Table;
use bevy::ecs::world::unsafe_world_cell::UnsafeWorldCell;
use bevy::prelude::Component;

pub type Copied<T> = ModifiedQ<CopiedQ<T>>;
pub type AsDeref<T> = ModifiedQ<AsDerefQ<T>>;
pub type OrDefault<T> = ModifiedQ<OrDefaultQ<T>>;
pub type OrDefaultOfDeref<T> = OrDefault<AsDeref<T>>;
pub struct OrBool<const V: bool, T>(PhantomData<T>);
pub struct OrUsize<const V: usize, T>(PhantomData<T>);
pub struct OrU32<const V: u32, T>(PhantomData<T>);


pub type AsDerefMut<T> = ModifiedQMut<AsDerefMutQ<T>>;

pub struct ModifiedQ<T>(PhantomData<T>);
pub struct ModifiedQMut<T>(PhantomData<T>);

pub struct CopiedQ<T>(PhantomData<T>);
pub struct AsDerefQ<T>(PhantomData<T>);
pub struct AsDeref2Q<T>(PhantomData<T>);
pub struct AsDerefMutQ<T>(PhantomData<T>);
pub struct OrDefaultQ<T>(PhantomData<T>);


pub trait ModifiedQuery {
    type FromQuery: ReadOnlyWorldQuery;
    type ModItem<'q>;

    fn modify_reference<'s>(from: <Self::FromQuery as WorldQuery>::Item<'s>) -> Self::ModItem<'s>;

    fn shrink<'wlong: 'wshort, 'wshort>(item: Self::ModItem<'wlong>) -> Self::ModItem<'wshort>;
}

pub trait ModifiedQueryMut {
    type FromQuery: WorldQuery;
    type ModItem<'q>; 
    type ReadOnly: ReadOnlyWorldQuery<State = <<Self as ModifiedQueryMut>::FromQuery as WorldQuery>::State>;

    fn modify_reference<'s>(from: <Self::FromQuery as WorldQuery>::Item<'s>) -> Self::ModItem<'s>;

    fn shrink<'wlong: 'wshort, 'wshort>(item: Self::ModItem<'wlong>) -> Self::ModItem<'wshort>;
}



impl <T: Component + Copy> ModifiedQuery for CopiedQ<T> {
    type FromQuery = &'static T;
    type ModItem<'a> = T;

    fn modify_reference<'s>(t: <Self::FromQuery as WorldQuery>::Item<'s>) -> Self::ModItem<'s> {
        *t
    }

    fn shrink<'wlong: 'wshort, 'wshort>(item: Self::ModItem<'wlong>) -> Self::ModItem<'wshort> {
        item
    }
}

impl <T: ReadOnlyWorldQuery> ModifiedQuery for OrDefaultQ<T> 
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

impl <T: Component + Deref> ModifiedQuery for AsDerefQ<T> {
    type FromQuery = &'static T;
    type ModItem<'a> = &'a <T as Deref>::Target;

    fn modify_reference<'s>(t: <Self::FromQuery as WorldQuery>::Item<'s>) -> Self::ModItem<'s> {
        t.deref()
    }

    fn shrink<'wlong: 'wshort, 'wshort>(item: Self::ModItem<'wlong>) -> Self::ModItem<'wshort> {
        item
    }
}

impl <T: Component + DerefMut> ModifiedQueryMut for AsDerefMutQ<T> {
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



impl <T: ReadOnlyWorldQuery, const V: bool> ModifiedQuery for OrBool<V, T> 
    where for<'a> <T as WorldQuery>::Item<'a>: Borrow<bool> {
    type FromQuery = Option<T>;
    type ModItem<'s> = bool;

    fn modify_reference<'s>(t: <Self::FromQuery as WorldQuery>::Item<'s>) -> Self::ModItem<'s> {
        t.map(|b|*b.borrow()).unwrap_or(V)
    }

    fn shrink<'wlong: 'wshort, 'wshort>(item: Self::ModItem<'wlong>) -> Self::ModItem<'wshort> {
        item
    }
}

impl <T: ReadOnlyWorldQuery, const V: usize> ModifiedQuery for OrUsize<V, T> 
    where for<'a> <T as WorldQuery>::Item<'a>: Borrow<usize> {
    type FromQuery = Option<T>;
    type ModItem<'s> = usize;

    fn modify_reference<'s>(t: <Self::FromQuery as WorldQuery>::Item<'s>) -> Self::ModItem<'s> {
        t.map(|b|*b.borrow()).unwrap_or(V)
    }

    fn shrink<'wlong: 'wshort, 'wshort>(item: Self::ModItem<'wlong>) -> Self::ModItem<'wshort> {
        item
    }
}

impl <T: ReadOnlyWorldQuery, const V: u32> ModifiedQuery for OrU32<V, T> 
    where for<'a> <T as WorldQuery>::Item<'a>: Borrow<u32> {
    type FromQuery = Option<T>;
    type ModItem<'s> = u32;

    fn modify_reference<'s>(t: <Self::FromQuery as WorldQuery>::Item<'s>) -> Self::ModItem<'s> {
        t.map(|b|*b.borrow()).unwrap_or(V)
    }

    fn shrink<'wlong: 'wshort, 'wshort>(item: Self::ModItem<'wlong>) -> Self::ModItem<'wshort> {
        item
    }
}

// Implementing WorldQuery/ReadOnlyWorldQuery

unsafe impl <T: ModifiedQuery> WorldQuery for ModifiedQ<T> {
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
unsafe impl <T: ModifiedQuery> ReadOnlyWorldQuery for ModifiedQ<T> {}

unsafe impl <T: ModifiedQueryMut> WorldQuery for ModifiedQMut<T> {
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
//*/