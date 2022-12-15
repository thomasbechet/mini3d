// Vertex Shader

struct GlobalData {
    canvas_resolution: vec2<u32>,
};
@group(0) @binding(0)
var<uniform> global: GlobalData;

struct InstanceData {
    color: vec4<f32>,
};
@group(0) @binding(1)
var<uniform> instances: array<InstanceData, 256>;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_main(
    @location(0) position_depth: vec3<f32>,
    @location(1) uv: vec2<f32>,
    @builtin(instance_index) instance_index: u32,
) -> VertexOutput {
    var out: VertexOutput;
    let pos = (position.xy / global.canvas_resolution) * 2.0 - 1.0;
    out.clip_position = vec4<f32>(pos, position_depth.z, 1.0);
    out.uv = uv;
    return out;
}

// Fragment shader

@group(0) @binding(2)
var s_texture: sampler;
@group(0) @binding(3)
var t_texture: texture_2d<f32>;

@fragment
fn fs_main(
    in: VertexOutput,
    @builtin(instance_index) instance_index: u32,
) -> @location(0) vec4<f32> {
    let filter = instances[instance_index].color;
    let color = textureSample(t_texture, s_texture, vec2<f32>(in.uv.x, 1.0 - in.uv.y)); 
    return color * filter;
}