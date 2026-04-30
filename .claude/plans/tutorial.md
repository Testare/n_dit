# Tutorial Plan

Work-in-progress plan for completing the tutorial system.

## What's been done

- `TutorialPlugin` — core infrastructure: `TutorialState`, `InTutorial`, `Tutorial`, `TutorialId`, `TutorialIndication` event, and the `tutorial_indicate` yarn command binding
- `TutorialOp` trait — `NodeOp` implements it, restricting player actions to those allowed by the current `TutorialState`
- Tutorial scene (`assets/nightfall/nodes/tutorial.scn.ron`) — basic node layout with `InTutorial`, `TutorialState`, `TutorialId` on entities
- Tutorial dialogue (`assets/dialogue/nf/initial.yarn`) — full script written, though most in-game triggers are still stubs
- `tutorial_ui.rs` — skeleton in place (`TutorialUiPlugin` registered, shader file written)
- `nf.rs` — when the player enters the tutorial node, dialogue starts on the `"tutorial"` yarn node
- `sys_dialog_response_to_node_op` — handles `[trigger name="..." /]` markup to advance dialogue from game ops
- `node_ui_op.rs` — fires `"tutorial::node_ui::move_cursor::({x}, {y})"` trigger events on cursor movement

## What still needs to be done

### 1. `TutorialState` advancement via yarn commands
`TutorialState` is hardcoded in the scene as `["node", "move", "e"]` and never updated. Since tutorials are dialog-centric flows, yarn drives advancement: implement a `<<set_tutorial_state "..." "..." "...">>` yarn command (or similar) that mutates `TutorialState` on the player's tutorial. The scene's initial state should reflect the first step; each subsequent yarn node advances it before gating the player's next action.

### 2. Implement `sys_add_tutorial_ui_flags_on_tutorial_entered` and `TutorialIndicatorName`
Add a `TutorialIndicatorName(String)` component (in `game_core::op::tutorial`) as a simple marker with no Entity field — intended for dynamically-spawned **UI entities** that can't be pre-labeled in scene files. When a player enters a tutorial node, `sys_add_tutorial_ui_flags_on_tutorial_entered` (in `cq_term`) scans UI entities with `TutorialIndicatorName`, gets the tutorial entity from `InTutorial` on the player (using `ForPlayer` to scope to the right player), and uses them to create `TutorialId(name, tutorial_entity)` components so `bind_tutorial_indicate` can find them. `TutorialIndicatorName` is left in place so the system works correctly if the player re-enters the node.

Note: scene entities (access points, curios in `.scn.ron`) don't need `TutorialIndicatorName` — they can carry `TutorialId` directly in the scene file since the Tutorial entity ID is static and `#[entities]` handles remapping on load.

`TutorialIndication` fires with the **scene/game entity** (whichever has `TutorialId` in the scene file). The UI layer is responsible for mapping scene entity → UI entity when it receives the event — `sys_react_to_tutorial_indication_events` does the lookup to find the screen position. Moving `TutorialId` from scene entities to UI entities at load time would require awkward cross-layer logic and isn't worth it.

### 3. Name all indicatable tutorial entities
The dialogue references many named entities beyond `lower_access_point` (hack program, upper access point, slingshot, etc.). Each game-logic entity needs `TutorialId` in the scene; each corresponding UI entity needs `TutorialIndicatorName` in its spawning code.

### 4. Wire dialogue-to-game-action triggers for remaining tutorial steps
Most tutorial steps in `initial.yarn` use backtick-wrapped text (e.g. `` `click on Hack` ``) which is only display text with no trigger. Only one step has a proper `[trigger name="tutorial::node_ui::move_cursor::(3, 4)" /]`. The remaining steps need proper `[trigger]` markup or `<<tutorial_indicate ...>>` calls tied to real game actions.

### 5. Tutorial visual indicator
In `tutorial_ui.rs`, `MaterialChPlugin::<TutorialIndicator>` and `TheTutorialIndicatorHandle` are commented out. `sys_react_to_tutorial_indication_events` only logs a debug message instead of showing the indicator on screen. The `tutorial_indicator.wgsl` shader is written but never used. This whole subsystem needs to be wired up.

### 6. Implement `enter_tutorial` / `exit_tutorial` yarn commands
The dialogue calls `<<enter_tutorial>>` and `<<exit_tutorial>>` but no Rust system handles them. `sys_yarn_commands` in `dialog.rs` only handles `open_shop`. These need to be registered and wired to the appropriate game transitions (enter/quit the tutorial node, or toggle tutorial state).

---

## Design interpretation

### Intended architecture

The tutorial system is designed around a strict `game_core` / `cq_term` separation, consistent with the rest of the codebase:

**`game_core::op::tutorial`** owns all tutorial logic:
- `TutorialState` — a list of strings encoding what action is currently expected (e.g. `["node", "move", "e"]`). `TutorialOp` implementations on game ops validate against this before allowing execution. **Yarn commands advance `TutorialState`** — since tutorials are dialog-centric, the yarn script calls e.g. `<<set_tutorial_state "node" "load" ...>>` at each step rather than a Bevy system auto-detecting completed ops.
- `InTutorial` / `Tutorial` relationships — track which player entities are in a tutorial and which `Tutorial` entity governs them.
- `TutorialIndicatorName(String)` — a simple marker component (no Entity field) for dynamically-spawned **UI entities** that can't carry `TutorialId` at spawn time. `sys_add_tutorial_ui_flags_on_tutorial_entered` uses these to create `TutorialId` components at tutorial-entry time (the `TutorialIndicatorName` is left in place — UI may be reused across node visits).
- `TutorialId(String, Entity)` — indicator tag scoped to a specific tutorial by the Tutorial entity. On **scene entities** it lives in the `.scn.ron` directly (static Tutorial entity ID, remapped by `#[entities]` on load). On **UI entities** it's attached dynamically from `TutorialIndicatorName` by `sys_add_tutorial_ui_flags_on_tutorial_entered`.
- `TutorialIndication` event — fired when a named entity should be visually highlighted. Contains the resolved UI entity ID.
- `tutorial_indicate` yarn command — bridges the dialogue script into the game: when yarn calls `<<tutorial_indicate "lower_access_point">>`, a one-shot system (bound per-player) resolves the Tutorial entity from `InTutorial`, searches for `TutorialId("lower_access_point", tutorial_entity)` on UI entities, and fires `TutorialIndication(ui_entity)`.

**`cq_term::tutorial_ui`** handles all presentation:
- Reacts to `TutorialIndication` events and renders the tutorial indicator shader (`tutorial_indicator.wgsl`) over the highlighted entity's screen position.
- `sys_add_tutorial_ui_flags_on_tutorial_entered` — the bridging system. When a player enters a tutorial node (detected via `InTutorial`), scans UI entities with `TutorialIndicatorName`, gets the tutorial entity from the player's `InTutorial`, and creates `TutorialId(name, tutorial_entity)` on those UI entities. `TutorialIndicatorName` is left in place so the system works correctly if the player re-enters the node.

### Entity naming: two-tier approach

**Scene/game entities** carry `TutorialId(name, tutorial_entity)` directly in `.scn.ron`. The Tutorial entity ID is static (known at scene-authoring time) and `#[entities]` remaps it to the runtime ID on load. No extra component needed.

**UI entities** are spawned dynamically and can't carry `TutorialId` at creation time. They get `TutorialIndicatorName(name)` in their spawning code; `sys_add_tutorial_ui_flags_on_tutorial_entered` (in `cq_term`) uses these to create `TutorialId(name, tutorial_entity)` components once the player's tutorial is known via `InTutorial`. `TutorialIndicatorName` is left in place so the system can re-run correctly if the player re-enters the node.

### Multiplayer design

Each player's node UI is separately keyed by `ForPlayer`, so their UI entities are distinct. `bind_tutorial_indicate` is a per-player closure that captures the specific player's entity ID, so it only resolves `TutorialId` entries for that player. Two players in simultaneous tutorials each get their own `TutorialIndication` events targeting their own UI entities. This should be sound.

### Design decisions (resolved)

1. **Scene-side naming**: Use `TutorialIndicatorName(String)` on scene entities rather than `TutorialId` with hardcoded entity IDs. The UI layer reads these at tutorial-entry time and produces `TutorialId(name, tutorial_entity)` on UI entities. This keeps the mapping data in the scene file without requiring runtime entity IDs there.

2. **`TutorialState` advancement**: Yarn commands drive it. The yarn script calls e.g. `<<set_tutorial_state "node" "load" ...>>` to unlock each next step. This is the natural fit for a dialog-centric flow and keeps tutorial pacing fully under the yarn author's control.
