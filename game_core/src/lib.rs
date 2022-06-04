#[macro_use]
extern crate lazy_static;

// Level 0
mod common;
mod error;
mod metadata;
// Level 1
mod abstractions;
// Level 2
mod model;
// Level 3
// Level 4
mod ai;
mod event;
// Level 5
// Level 6
mod game_master;

pub use common::{Bounds, Direction, Point, PointSet, GridMap};
// use event::{Event, GameEvent, EventSubtype, EventConstructor, EventErr};
use abstractions::StateChange;
pub use game_master::{AuthorityGameMaster, EventPublisher, GameCommand};
pub use metadata::Metadata;
pub use model::curio::{Curio, Team};
pub use model::curio_action::{CurioAction, StandardCurioAction};
pub use model::game_state::{GameChange, GameState};
pub use model::inventory::{Card, Inventory, Item, Pickup};
pub use model::node::{Node, NodeChange, Sprite};
pub use model::world_map::WorldMap;

use ai::EnemyAi;