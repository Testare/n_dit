@compute @workgroup_size(1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    charmi::view.cells[global_id.x] = charmi::GAP_CELL;
}
