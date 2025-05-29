#define_import_path charmi
#import bevy_render::globals::Globals;

struct CharmiCell {
    @location(0) ch: u32,
    @location(1) fg: u32,
    @location(2) bg: u32,
    @location(3) attr: u32,
}

struct CharmiImage {
    @location(0) width: u32,
    @location(1) height: u32,
    @location(2) cells: array<CharmiCell>
}

struct TransformCh {
    @location(0) position: vec3i,
    @location(1) scale: vec2u,
}

@group(0) @binding(0) var<storage, read_write> view: CharmiImage;
@group(0) @binding(1) var<uniform> view_transform: TransformCh;
@group(0) @binding(2) var<uniform> globals: Globals;
@group(1) @binding(0) var<uniform> transform: TransformCh;

const NO_COLOR = 0xFEu << 24;
const TRUE_COLOR = 0x01u << 24;
const SUPPRESSED_CHAR = 0xDFFFu;
const GAP_CELL = CharmiCell(0, NO_COLOR, NO_COLOR, 0);

// Returns position of cell in the space of the view
fn view_pos(global_id: vec3u) -> vec3u {
    let z = global_id.x;
    let x = z % view.width;
    let y = z / view.width;
    return vec3(x, y, z);
}

// Returns position of cell in space of the sprite
// Returns vec(-1, -1) if position is not on the sprite
fn sprite_pos(global_id: vec3u) -> vec2i {
    let local_pos = vec2(
        i32(global_id.x % view.width),
        i32(global_id.x / view.width)
    ) + view_transform.position.xy;

    let scale = vec2(i32(transform.scale.x), i32(transform.scale.y));

    if all(local_pos >= transform.position.xy & local_pos < transform.position.xy + scale) {
        return local_pos - transform.position.xy;
    } else {
        return vec2(-1, -1);
    }
}

// The position of cell in sprite space for view-locked sprites
// Returns vec(-1, -1) if position is not on the sprite
fn sprite_pos_view_locked(global_id: vec3u) -> vec2i {
    let local_pos = vec2(
        i32(global_id.x % view.width),
        i32(global_id.x / view.width)
    );

    let scale = vec2(i32(transform.scale.x), i32(transform.scale.y));

    if all(local_pos >= transform.position.xy & local_pos < transform.position.xy + scale) {
        return local_pos - transform.position.xy;
    } else {
        return vec2(-1, -1);
    }
}
