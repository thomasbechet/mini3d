// Vertex Shader

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>
};

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    var out: VertexOutput;
    let x: f32 = f32((vertex_index & 1u) << 2u);
    let y: f32 = f32((vertex_index & 2u) << 1u);
    out.uv.x = x * 0.5;
    out.uv.y = y * 0.5;
    out.clip_position = vec4<f32>(x - 1.0, y - 1.0, 0.0, 1.0);
    return out;
}

// Fragment Shader

@group(0) @binding(0)
var t_texture: texture_2d<f32>;
@group(0) @binding(1)
var s_texture: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(t_texture, s_texture, in.uv);
}