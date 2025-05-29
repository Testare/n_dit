#import charmi::{
    TRUE_COLOR,
    globals,
    sprite_pos,
    transform,
    view
}

@group(2) @binding(0) var<storage, read_write> textIn: array<u32>;

@compute @workgroup_size(1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let i = (global_id.x);
    let t = globals.frame_count;
    let p = sprite_pos(global_id);
    if p.x < 0 {
        return;
    }
    let x = p.x;
    let y = p.y;

    if x == 0 || y == 0 || x == i32(transform.scale.x) - 1 || y == i32(transform.scale.y) - 1 {
        let txy = (t + u32(x) + u32(y)) % 768;
        let shift = (txy/256)*8;
        view.cells[i].bg = ((txy % 256) << shift) | TRUE_COLOR;
        view.cells[i].ch = 0x20u;
    } else {
        let textlen = arrayLength(&textIn);
        let text_i = i - view.width - (i/view.width)*2 + 1;

        view.cells[i].bg = 0x00u;
        view.cells[i].fg = 0x07u;
        if text_i > textlen || textIn[text_i] == 0 {
            view.cells[i].ch = 0x20u;
        } else {
            view.cells[i].ch = textIn[text_i];
        }
    }
}
