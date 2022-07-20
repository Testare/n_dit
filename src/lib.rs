//! This crate currently contains all the logic for a game.
//!
//! This logic is divided into seperate parts:
//!
//! * [grid_map] contains the GridMap struct, which is a generic collection type struct, that is
//! built with a specific purpose in mind.
//! * [game] contains the core game logic: How the player's moves affect the world, the different
//! states, the map, etc.
//! * [ui] is the package that contains logic for how the player interfaces with the game. We plan
//! of there being several options here (CLI and terminal at least, and potentially GUI later).
//! Should contain absolutely no logic for how the game state is mutated.
//!
//! In the future, some of these will probably be separated into specific crates to avoid
//! unintentional coupling. As the code is updated, this decoupling should be ensured as much as
//! possible. The [ui] mod might also be separated into different crates for different interfaces.
//!
//! There is also the [hld] module, which is a dummy module to hold high-level design information.
//! We put this in a separate module to allow use of [mermaid.js] through [aquamarine], which can't
//! be done here because of internal macro attributes are unstable. [\[1\]]
//!
//!
//! | Module     | Responsibility                                                     |
//! |------------|--------------------------------------------------------------------|
//! | [game]     | Core logic of game, such as curios, points, money, etc.           |
//! | [ui]       | Everything between the user and the game, might be broken up later |
//! | [grid_map] | GridMap struct, a generic collection for use in game.              |
//!
//! [\[1\]]: https://github.com/rust-lang/rust/issues/5472
//! [mermaid.js]: https://mermaid-js.github.io/

#[cfg_attr(doc, aquamarine::aquamarine)]
///
///
/// GameState and UI are intentionally HEAVILY decoupled, in the future we hope to even have
/// separate crates for it. The hope is that we'll be able to create multiple UI's, that can
/// be used interchangeably for the same game with the same save file, or potentially a remotely
/// hosted game server in the future.
///
///```mermaid
///sequenceDiagram
///     actor user as User
///     participant ui as UI
///     participant game as Game State
///     user->>ui: User specifies desired action
///     game->>ui: UI reads state of game to determine what action means
///     ui->>game: If action affects game state, creates GameAction object and passes it to GameState.apply_action(..)
///     note right of game: game mutates based on action information
///     game->>ui: Returns result of action applied
///     ui->>user: Displays results to user
///```
///
/// Game Module heirarchy, ideally modules will only use modules they can link
/// to, directly or indirectly.
///
/// ```mermaid
/// flowchart RL
///     subgraph level0
///     common
///     end
///     subgraph level1
///     abstractions
///     end
///     subgraph level2
///     model
///     end
///     subgraph level3
///     changes
///     end
///     subgraph level4
///     ai
///     event
///     end
///     subgraph level5
///     command
///     end
///     subgraph level6
///     game_master
///     end
///     
///     abstractions --> common
///     model --> abstractions
///     changes --> model
///     ai --> changes
///     event --> changes
///     command --> event
///     game_master --> command
///     game_master --> ai
///     
///     
///
/// ```
///
pub mod hld {}

pub mod charmie_ui;
pub mod ui;
pub use game_core::GridMap;
pub use ui::*;
