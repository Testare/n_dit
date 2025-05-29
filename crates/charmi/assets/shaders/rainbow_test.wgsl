#import bevy_render::globals::Globals;

struct CharmiFormat {
    @location(0) ch: u32,
    @location(1) fg: u32,
    @location(2) bg: u32,
    @location(3) at: u32,
}

struct CharmiImage {
    @location(0) width: u32,
    @location(1) height: u32,
    @location(2) cells: array<CharmiFormat>
}

@group(0) @binding(0) var<storage, read_write> data: CharmiImage;
@group(0) @binding(1) var<storage, read_write> textIn: array<u32>;
@group(0) @binding(2) var<uniform> globals: Globals;

@compute @workgroup_size(1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let index = global_id.x;
    let x = index % data.width % 256;
    let y = index / data.width % 256;
    let t = globals.frame_count % 256;
    data.cells[index].at = 1u;
    data.cells[index].bg = (x + y + t) % 256;
    data.cells[index].ch = 32u;
}

fn translateCharmiToAnsi() {
    // ESC[38;x;
    // 256: c;5;n
    // true: c;2;r;g;b
    // fg: c = 38
    // bg: c = 48
    // ESC[38;x;<fg>;48;x;<bg>;]
    // Each line starts with ESC[<line>;0H. They might need to end with reseting colors but I'm not sure.
    // Can I also have work items that check if the line buffer is unchanged and prevents it from being written out?
    // Each work item can check if the cell is the same as before, and if so, perhaps we can skip it entirely
    // ​
    // On one hand, the bigger the buffer, the more data has to be read back, which takes time.
    // On the other hand,
}

