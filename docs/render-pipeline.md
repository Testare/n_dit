# Render Pipeline

n_dit renders a full game UI to the terminal using a GPU-accelerated pipeline. The chain runs every frame: game state changes produce `TerminalRendering` components holding `CharmiImage` data, which flow through a Taffy layout pass, get composited via GPU compute shaders, and are read back to the CPU for ANSI output.

## Pipeline Stages

```
Game Systems (update TerminalRendering)
        |
        v
  Layout (Taffy)            -- cq_term/src/layout.rs
        |
        v
  Sprite Sync               -- cq_term/src/render.rs
        |
        v
  GPU Compute Shaders       -- charmi/src/render/
        |
        v
  Readback + ANSI Output    -- charmi_core (CharmiImage::write_out_ansi)
        |
        v
  Terminal (stdout)
```

### 1. TerminalRendering

Any entity that should appear on screen gets a `TerminalRendering` component (defined in `cq_term/src/render.rs`). This holds a `CharmiImage` -- a 2D grid of `CharCell` values (character + foreground + background + attributes). Game UI systems build these images each frame using `CharmiBuilder`.

### 2. Layout (Taffy)

The layout system in `cq_term/src/layout.rs` maps Bevy's entity hierarchy onto the Taffy layout engine.

Key components:
- `StyleTty` -- wraps `taffy::Style`, drives flexbox/grid layout
- `LayoutRoot` -- marks the root of a render tree, fitted to terminal size
- `CalculatedSizeTty` -- computed width/height after layout
- `GlobalTranslationTty` -- computed screen position (x, y)
- `VisibilityTty` -- controls whether the entity renders
- `RenderOrder` -- z-depth for layering

The layout plugin runs these systems in order:
1. `taffy_new_style_components` -- create Taffy nodes for new `StyleTty` entities
2. `taffy_apply_style_updates` -- sync changed styles to Taffy
3. `taffy_apply_hierarchy_updates` -- mirror Bevy parent-child into Taffy
4. `calculate_layouts` -- run Taffy layout, write `CalculatedSizeTty` and `GlobalTranslationTty`
5. `render_layouts` -- composite children into parent `TerminalRendering` images

### 3. Sprite Sync

`sys_update_charmi_sprites()` in `cq_term/src/render.rs` bridges the layout world and the GPU render world:
- Copies `TerminalRendering` data into `CharmiImageSprite` (the GPU-side component)
- Sets `TransformCh.position` from `GlobalTranslationTty` + `RenderOrder`
- Sets `TransformCh.scale` from `CalculatedSizeTty` (or zero if invisible)

### 4. GPU Compute Rendering

The GPU pipeline lives in `charmi/src/render/`. It uses Bevy's render graph with compute shaders (WGSL) rather than traditional fragment shaders.

Key types:
- **`ViewCh`** (`render/view.rs`) -- the render target, a `ShaderStorageBuffer` representing the terminal's character grid
- **`CharmiImageSprite`** (`render/image_sprite.rs`) -- GPU-side sprite data synced from `TerminalRendering`
- **`TransformCh`** (`render/transform.rs`) -- position (IVec3 with z-depth) and scale (UVec2), uploaded as `DynamicUniformBuffer`
- **`RenderLayer`** (`render/layer.rs`) -- groups sprites for rendering; `RenderedBy` links a sprite to its view
- **`CharmiPhase`** -- sorted render items, ordered by z-depth

The render graph node `MainPassChNode` runs each frame:
1. Clear the `ViewCh` buffer using the `view_clear.wgsl` compute shader
2. For each `CharmiPhaseItem` (sorted by z-order), dispatch the `image_sprite.wgsl` compute shader
3. Each dispatch runs one workgroup per cell in the destination buffer

Embedded shaders:
- `view_clear.wgsl` -- zero out the render target
- `image_sprite.wgsl` -- composite a sprite onto the view buffer
- `charmi.wgsl` -- core character cell definitions
- `box.wgsl` -- box-drawing character generation

### 5. GPU Readback and ANSI Output

After the compute pass, Bevy's `Readback::buffer()` transfers the `ViewCh` storage buffer back to the CPU. A `ReadbackComplete` observer fires when the transfer finishes:

1. Convert the raw buffer into a `CharmiImage` via `CharmiImage::from(&ReadbackComplete)`
2. Diff against the previous frame's `RenderCache` (cached `CharmiImage`)
3. Call `CharmiImage::write_out_ansi(stdout, render_cache)`:
   - Compare each line against the cached image
   - For changed lines, emit ANSI escape sequences: `MoveTo`, `SetForegroundColor`, `SetBackgroundColor`, `Print(char)`, `ResetColor`
   - Only changed lines are rewritten (the primary optimization)
4. Store the new image in `RenderCache` for next frame

### System Set Ordering

All render systems run in `Update`, organized by `RenderTtySet`:

```
AdjustLayoutStyle
    -> PreCalculateLayout
        -> CalculateLayout        (Taffy runs here)
            -> PostCalculateLayout
                -> RenderLayouts   (composites children into parents)
                    -> RenderToTerminal  (syncs to GPU sprites)
```

The GPU compute pass and readback happen in Bevy's render schedule, after the main `Update`.

## Terminal I/O

`TerminalWindow` (in `cq_term/src/lib.rs`) manages the terminal state:
- On startup: enters alternate screen, hides cursor, enables mouse capture
- On shutdown (or panic): restores cursor, leaves alternate screen, disables mouse capture
- Tracks terminal dimensions; layout roots are fitted to this size

Input flows through a separate thread (`TermEventListener`) that polls crossterm events and sends them into the ECS via channels, dispatched as `CrosstermEvent`, `KeyEvent`, and `MouseEventTty` in the `First` schedule.
