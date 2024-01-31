use std::marker::PhantomData;

use bevy::hierarchy::BuildWorldChildren as _;
use bevy::reflect::TypePath;

use crate::prelude::*;

#[derive(Component, Debug, Deref, Resource)]
pub struct Daddy<T> {
    #[deref]
    entity: Entity,
    _phantom_data: PhantomData<fn(T) -> T>, // Invariant, but Sync+Send
}

/// We don't really have T
unsafe impl<T> Send for Daddy<T> {}
unsafe impl<T> Sync for Daddy<T> {}

#[derive(Component, Debug, Deref, Resource)]
pub struct Daddies(Entity);

impl FromWorld for Daddies {
    fn from_world(world: &mut World) -> Self {
        let id = world.spawn(Name::new("Daddies")).id();
        world.entity_mut(id).insert(Daddies(id));
        Daddies(id)
    }
}

impl<T: TypePath> FromWorld for Daddy<T> {
    fn from_world(world: &mut World) -> Self {
        world.init_resource::<Daddies>();
        let &Daddies(daddies_id) = world.resource::<Daddies>();
        let entity = world
            .spawn(Name::new(format!("Daddy [{}]", T::type_path())))
            .id();
        world.entity_mut(entity).insert(Self {
            entity,
            _phantom_data: PhantomData::<fn(T) -> T>,
        });
        world.entity_mut(daddies_id).add_child(entity);
        Daddy {
            entity,
            _phantom_data: PhantomData::<fn(T) -> T>,
        }
    }
}
