struct GlobalData {
    resolution: vec2<u32>,
};
@group(0) @binding(0)
var<uniform> global: GlobalData;

// Vertex Shader

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) @interpolate(flat) color: vec4<f32>,
};

@vertex
fn vs_main(
    @location(0) pos: vec2<i32>,
    @location(1) depth: f32,
    @location(2) color: vec4<f32>,
) -> VertexOutput {
    var out: VertexOutput;

    // Extract position
    var position = (vec2<f32>(pos) + 0.25) / vec2<f32>(global.resolution);
    position.y = 1.0 - position.y;

    // Normalize position and save color
    out.clip_position = vec4<f32>(position * 2.0 - 1.0, depth, 1.0);
    out.color = color;
    
    return out;
}

// Fragment shader

@fragment
fn fs_main(
    in: VertexOutput,
) -> @location(0) vec4<f32> {
    return in.color;
}