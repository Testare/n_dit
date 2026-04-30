# Game Architecture

n_dit is a Bevy ECS game that recreates "The Nightfall Incident" as a terminal UI application. This document covers the high-level architecture: how the crates fit together, how a frame flows through the system, and the key design decisions.

## Crate Dependency Graph

```
n_dit (binary)
  |
  +-- cq_term (terminal UI)
  |     |
  |     +-- game_core (game logic)
  |     +-- charmi (Bevy rendering)
  |           |
  |           +-- charmi_core (data model)
  |                 |
  |                 +-- charmi_macro (proc macro DSL)
  |
  +-- game_core
  +-- charmi
```

`game_core` has no rendering dependencies. `cq_term` bridges game logic and terminal display. `charmi` is the rendering library. `n_dit` wires everything together as a Bevy app.

## Frame Pipeline

Every frame passes through ordered system sets defined by `NDitCoreSet`:

```
First:       RawInputs          -- poll crossterm events from background thread
PreUpdate:   ProcessInputs      -- translate raw events to game inputs
Update:      ProcessCommands    -- execute queued operations (NodeOp, SaveOp, etc.)
             ProcessCommandsFlush
             PostProcessCommands
             ProcessUiOps       -- execute UI operations (shop, dialogue, etc.)
             PostProcessUiOps
Update:      RenderTtySet::*    -- layout, composite, and render to terminal
```

Game logic runs in `ProcessCommands`. UI reactions run in `ProcessUiOps`. Rendering runs last in `Update`.

## Plugin Structure

The `n_dit` binary assembles these plugins:

**Game logic** (`NDitCorePlugin` in `game_core/src/lib.rs`):
- `BoardPlugin`, `CardPlugin`, `NodePlugin`, `PlayerPlugin`
- `EntityGridSupportPlugin`, `DialogPlugin`, `SavePlugin`
- `ItemPlugin`, `QuestPlugin`, `ShopPlugin`, `RegistryPlugin`
- `OpExecutorPlugin::<CoreOps>` -- processes game operations

**Terminal UI** (`CharmiePlugin` in `cq_term/src/lib.rs`):
- `TaffyTuiLayoutPlugin` -- flexbox layout via Taffy
- `RenderTtyPlugin` -- rendering pipeline
- `CharmiRenderPlugin` -- GPU compute shaders
- Animation, input, and UI module plugins

## Key Patterns

### Operation-Driven State Changes

All game state mutations go through the Op system (see [operation-system.md](operation-system.md)). Input systems never modify game state directly -- they queue operations into `CoreOps` (for game logic) or `UiOps` (for UI state). This creates a clean audit trail and enables future networked play.

### UI/Logic Separation

`game_core` defines components and systems with no awareness of how they'll be displayed. `cq_term` reads game state and produces `TerminalRendering` components. This separation means a hypothetical GUI frontend would only replace `cq_term`.

### Charmi as the Rendering Primitive

Everything visible on screen is a `CharmiImage` -- a 2D grid of styled characters. UI systems build these images using `CharmiBuilder`, attach them as `TerminalRendering` components, and the render pipeline (see [render-pipeline.md](render-pipeline.md)) composites them to the terminal.

## Node Gameplay

A "node" is a puzzle/battle encounter. When the player enters a node:

1. `NodeOp::EnterNode` sets up the `EntityGrid`, teams, and access points
2. **Setup phase**: player deploys cards from their `Deck` to `AccessPoint`s, creating `Curio` entities on the grid
3. **Play phase**: turn-based gameplay via `NodeOp` (move curios, perform actions, end turns)
4. **Victory/defeat**: `VictoryStatus` is set on each team; pickups are granted on exit

Curios are the game pieces. Each curio is backed by a `Card` (defining stats, actions, movement speed, max size). Cards live in the player's `Deck` and are deployed to the grid during setup.

## Dialogue

Dialogue uses bevy_yarnspinner with `.yarn` story files. The `Dialog` component tracks current line and options. Game operations can trigger dialogue advancement via `DialogTrigger` events, and yarn commands can trigger game operations (e.g., `open_shop`). This bidirectional link keeps narrative and gameplay loosely coupled.

## Save/Load

Saving and loading use dedicated Bevy schedules (`SaveSchedule` / `LoadSchedule`). Each subsystem hooks into these schedules to persist and restore its own state. Save data is written as JSON to a file in `~/.local/share/nf/` (or a user-specified path).
