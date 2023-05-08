use bevy::prelude::*;

#[derive(Component, Reflect)]
struct Node {}

#[derive(Component, Reflect)]
struct GridMap {
    // Re-implement grid_map.rs to store entity references directly
}

#[derive(Component, Reflect)]
struct NodePiece {
    display_name: String,
}

#[derive(Component, Reflect)]
struct Mon {
    value: u32,
}

#[derive(Component, Reflect)]
struct AccessPoint {
    card: Entity, // Display card data to load
}

#[derive(Component, Reflect)]
struct Card {
    curio_actions: Vec<Entity>, // Entities or just a list of them directly?
    tags: Vec<Tag>,
}

#[derive(FromReflect, Reflect)]
enum Tag {
    Fire,
    Flying,
}

#[derive(Component, Reflect)]
struct Curio {
    max_size: usize,
    speed: usize,
    owner: Entity,
    card: Entity,
}

#[derive(Component, Reflect)]
struct CurioAction {}
