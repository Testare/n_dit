#import bevy_render::globals::Globals;

struct CharmiCell {
    @location(0) ch: u32,
    @location(1) fg: u32,
    @location(2) bg: u32,
    @location(3) at: u32,
}

struct CharmiImage {
    @location(0) width: u32,
    @location(1) height: u32,
    @location(2) cells: array<CharmiCell>
}

// This is the data that lives in the gpu only buffer
// @group(0) @binding(0) var<uniform> globals: Globals;
@group(0) @binding(0) var<storage, read_write> data: CharmiImage;
@group(0) @binding(1) var<storage, read_write> textIn: array<u32>;
@group(0) @binding(2) var<uniform> globals: Globals;

@compute @workgroup_size(1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    // We use the global_id to index the array to make sure we don't
    // access data used in another workgroup
    let x = global_id.x % data.width;
    let y = global_id.x / data.width;
    let old_fg = data.cells[global_id.x].fg;
    let i = x - globals.frame_count/7;
    data.cells[global_id.x].at = 1u;
    let textlen = arrayLength(&textIn);
    let textlen_spaced =textlen + 5;
    data.cells[global_id.x].fg = (i/textlen_spaced + y) % 256u;//(data.cells[global_id.x].fg + 1u) % 256u;
    if i % textlen_spaced >= textlen {
        data.cells[global_id.x].ch = 32u;
    } else {
        data.cells[global_id.x].ch = textIn[i % textlen_spaced];
    }
    // Write the same data to the texture
    // textureStore(texture, vec2<i32>(i32(global_id.x), 0), vec4<u32>(data[global_id.x].ch, 0, 0, 0));
}
