# Operation System

All game actions in n_dit flow through a centralized operation (Op) system. This provides a single bottleneck for game state mutations, which makes the game deterministic, replayable, and ready for future multiplayer (where operations could be serialized over a network).

## How It Works

```
Player input / AI decision
        |
        v
  Queue an Op into OpExecutor
        |
        v
  OpExecutorPlugin runs sys_perform_ops()
        |
        v
  Registered system executes the Op
        |
        v
  OpResult<O> event emitted (with Metadata or OpError)
        |
        v
  UI systems react to OpResult events
```

## Core Types

### `Op` trait (`game_core/src/op.rs`)

Every operation implements this trait:
```rust
pub trait Op: Send + Sync + 'static {
    fn system_index(&self) -> usize;           // Which handler to run
    fn register_systems(registrar: &mut ...);  // Register all handlers
    fn to_request(self, source: Entity) -> OpRequest;
}
```

Each variant of an Op enum maps to a `system_index`, which selects the registered system function to execute it.

### `OpExecutor` (`game_core/src/op/executor.rs`)

A queue of pending operations:
- `Local(Vec<OpRequest>)` -- single-player queue
- `Network` -- placeholder for future multiplayer

Methods:
- `request(source, op)` -- enqueue an operation
- `take_ops()` -- drain all pending operations for processing

### `OpResult<O>` (Event)

Emitted after each operation executes:
- `source: Entity` -- who initiated it
- `op: O` -- the operation that ran
- `result: Result<Metadata, OpError>` -- outcome

### `Metadata` (`game_core/src/common/metadata.rs`)

A typed key-value store (`HashMap<String, String>` with JSON-serialized values) returned from successful operations. Downstream systems read specific keys to react -- for example, a UI system reads the `CURIO` key from a movement result to animate the piece.

### `OpError`

Four severity levels:
- `InvalidOp` -- bad input, no state changed
- `OpFailureRecoverable` -- partial failure, can retry
- `OpFailureCritical` -- unrecoverable game error
- `FrameworkError` -- system-level error

## Game Operations

### `CoreOps` (Resource)

Wraps the `OpExecutor` for the main game operation types. Processed in `NDitCoreSet::ProcessCommands` during `Update`.

### `NodeOp` (`game_core/src/node/node_op.rs`)

All in-node gameplay actions:
- `MoveActiveCurio { dir }` -- move a piece in a cardinal direction
- `PerformCurioAction { action_id, curio, target }` -- use an ability
- `ActivateCurio { curio_id }` -- select a piece
- `LoadAccessPoint / UnloadAccessPoint` -- deploy or withdraw a card
- `ReadyToGo` -- signal setup is complete
- `EndTurn` -- end the current turn
- `EnterNode / QuitNode` -- enter or leave a node
- `Undo` -- revert last action

### `SaveOp` (`game_core/src/saving.rs`)

- `Save` -- serialize game state to file
- `Load` -- deserialize and restore

### `UiOps`

A separate `OpExecutor` for UI-layer operations (shop interactions, dialogue choices, etc.), processed in `NDitCoreSet::ProcessUiOps`.

## Example Flow: Moving a Piece

1. Player presses `w` -> `KeyEvent` dispatched
2. `KeyMap` translates to `NamedInput::Direction(North)`
3. Node input system queues `NodeOp::MoveActiveCurio { dir: North }` into `CoreOps`
4. `sys_perform_ops::<CoreOps>` runs `opsys_node_movement()`
5. System validates the move, updates `EntityGrid`, returns `Metadata` with position info
6. `OpResult<NodeOp>` event fires
7. UI systems read the result and update animations/rendering
