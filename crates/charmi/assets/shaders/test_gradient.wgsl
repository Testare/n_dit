[[stage(vertex)]]
fn vs_main(
    [[location(0)]] a_position: vec2<f32>,
) -> [[builtin(position)]] vec4<f32> {
    return vec4<f32>(a_position * 2.0 - 1.0, 0.0, 1.0);
}

[[stage(fragment)]]
fn fs_main() -> [[location(0)]] vec4<f32> {
    // Generate a simple gradient
    let frag_coord = vec2<f32>(gl_FragCoord.xy) / vec2<f32>(gl_FragCoord.zw);
    return vec4<f32>(frag_coord.x, frag_coord.y, 0.0, 1.0);
}
