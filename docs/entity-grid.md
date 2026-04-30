# Entity Grid

The entity grid (`game_core/src/entity_grid.rs`) is the spatial data structure that tracks where game pieces (curios) are on the game board during node gameplay. It maps entities to grid positions and supports multi-square entities through an internal linked list.

## Data Structure

```rust
pub struct EntityGrid {
    width: u32,
    height: u32,
    entries: HashMap<Entity, UVec2>,      // Entity -> head position
    grid: Vec<Vec<Option<Square>>>,       // 2D array of squares
}

pub struct Square {
    item: Option<Entity>,     // Which entity occupies this square
    next: Option<UVec2>,      // Next square in linked list (for multi-square entities)
    location: UVec2,
}
```

A `None` square means the cell is **closed** (wall/obstacle). A `Some(Square)` with `item: None` is an open, empty cell.

## Multi-Square Entities

Curios can occupy multiple cells. The grid tracks this as a linked list:
- `entries` maps an entity to its **head** position
- Each `Square` has a `next` pointer to the next square belonging to the same entity
- `push_back` / `push_front` add squares to an entity's chain
- `pop_back` / `pop_front` remove squares

This supports snake-like movement where a curio's size (from `MaximumSize`) determines how many squares it occupies.

## Key Operations

- `put_item(point, entity)` -- place a new entity at a position
- `item_at(point)` -- query which entity is at a position
- `square_is_free(point)` / `square_is_occupied(point)` / `square_is_open(point)` / `square_is_closed(point)` -- cell status checks
- `point_map(entity)` / `point_vec(entity)` -- get all positions an entity occupies
- `squares(entity)` / `squares_mut(entity)` -- iterate over an entity's linked squares

## Board Shape Serialization

The grid shape (which cells are open vs closed) is serialized to a base64-encoded bitstring via `shape_string_base64()` and restored via `from_shape_string()`. This compact encoding stores the grid topology in scene files without listing every cell.

## Relationship to Board

`EntityGrid` is a component on the same entity as the `Board` component. The `Board` provides identity and metadata while the `EntityGrid` provides the spatial state. During node gameplay, the operation system (`NodeOp`) manipulates the grid -- moving curios, checking collisions, and validating positions.
