# Charmi: Character Map Images

Charmi is n_dit's library for representing and rendering 2D images made of terminal characters. It spans three crates that handle data representation, Bevy integration, and a proc macro DSL.

## Crate Structure

- **`charmi_core`** -- Data model and CPU-side API (no Bevy dependency)
- **`charmi`** -- Bevy plugin with asset loaders and GPU render pipeline
- **`charmi_macro`** -- Proc macro for defining images in TOML syntax

## Core Data Model

### CharCell (`charmi_core/src/model/char_cell.rs`)

The atomic unit -- one terminal cell:
```rust
pub struct CharCell {
    pub ch: u32,    // Unicode codepoint (0 = GAP/transparent, 0x20 = BLANK/space)
    pub fg: u32,    // Foreground color
    pub bg: u32,    // Background color
    pub attr: u32,  // Text attributes (bold, underline, etc.)
}
```

Color encoding:
- `0xFE000000` (`NO_COLOR`) -- inherit/transparent
- `0x01RRGGBB` (`TRUE_COLOR`) -- 24-bit RGB
- `0-15` -- standard ANSI colors

Special handling: double-width characters (East Asian) automatically "crack" the next cell using `SUPPRESSED_CHAR` (0xDFFF).

### CharmiImage (`charmi_core/src/model/image.rs`)

A 2D grid of `CharCell` values:
```rust
pub struct CharmiImage {
    width: usize,
    height: usize,
    cells: Arc<[CharCell]>,  // Row-major, Arc for cheap cloning
}
```

Key operations:
- `build_sized(w, h)` / `build_fixed_width(w)` / `build_dynamic()` -- create via `CharmiBuilder`
- `clip(x, y, w, h)` -- extract a sub-region
- `draw(image, x, y)` -- composite another image on top (GAP cells are transparent)
- `write_out_ansi(writer, cache)` -- diff against cached frame and emit ANSI escape sequences

### CharmiAnimation and CharmiActor (`charmi_core/src/model/actor.rs`)

- `CharmiAnimation` -- sequence of `CharmiAnimationFrame`s with cumulative timings
- `CharmiActor` -- named collection of animations (e.g., "idle", "walk", "attack")
- Frame lookup via `image_for_timing(t)` -- finds the frame whose cumulative time covers `t`

## CharmiBuilder (`charmi_core/src/api/builder.rs`)

Fluent API for constructing images:
```rust
let image = CharmiImage::build_sized(10, 3)
    .fg(Color::Green)
    .add_text("Hello")
    .next_line()
    .fg(Color::Red)
    .add_text("World")
    .build();
```

Modes:
- **FixedSize** -- pre-allocated grid, writes into it
- **FixedWidth** -- fixed width, grows vertically
- **Dynamic** -- grows in both dimensions

## Proc Macro DSL (`charmi_macro`)

The `charmi_toml!()` macro lets you define images inline using TOML:

```rust
charmi_toml!(r#"
text = "HP: 100"
fg = "green"
bg = "black"
attr = "bold"
"#)
```

Supports dynamic expressions and value maps for per-character coloring. The macro parses TOML at compile time, validates color/attribute names, and generates `CharmiImage` construction code.

## Bevy Integration (`charmi/src/`)

### Asset Loaders

- `CharmiLoader` -- loads `.charmi` files (single images)
- `CharmiaLoader` -- loads `.charmia` files (actor definitions with multiple animations)

### GPU Render Pipeline

Charmi uses compute shaders rather than traditional rendering. See [render-pipeline.md](render-pipeline.md) for the full pipeline. The key GPU-side types:

- **`CharmiImageSprite`** -- GPU representation of a `CharmiImage`, synced from `TerminalRendering`
- **`ViewCh`** -- render target buffer representing the terminal grid
- **`TransformCh`** -- position and clipping rectangle on the render target

The compute shader `image_sprite.wgsl` composites each sprite onto the view buffer, handling transparency (GAP cells) and z-ordering.
