use bevy::prelude::*;

#[derive(Component, FromReflect, Reflect)]
struct Card;

#[derive(Component, Deref, FromReflect, Reflect)]
struct Tags {
    tags: Vec<Tag>,
}

#[derive(FromReflect, Reflect)]
enum Tag {
    Fire,
    Flying,
}

mod action {
    use bevy::prelude::*;

    #[derive(Component, FromReflect, Reflect)]
    struct Actions {
        actions: Vec<Entity>, // Entities or just a list of them directly?
    }

    #[derive(Component, FromReflect, Reflect)]
    struct Action {}
}
