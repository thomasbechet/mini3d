// Vertex Shader

struct GlobalData {
    world_to_clip: mat4x4<f32>,
};
@group(0) @binding(0)
var<uniform> global: GlobalData;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_normal: vec3<f32>,
    @location(1) uv: vec2<f32>,
};

@vertex
fn vs_main(
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) model_row0: vec4<f32>,
    @location(4) model_row1: vec4<f32>,
    @location(5) model_row2: vec4<f32>,
    @location(6) model_row3: vec4<f32>,
) -> VertexOutput {
    var out: VertexOutput;
    let model = mat4x4<f32>(
        model_row0,
        model_row1,
        model_row2,
        model_row3,
    );
    out.clip_position = global.world_to_clip * model * vec4<f32>(position, 1.0);
    out.world_normal = (model * vec4<f32>(normal, 0.0)).xyz;
    out.uv = uv;
    return out;
}

// Fragment shader

@group(0) @binding(1)
var s_texture: sampler;
@group(1) @binding(0)
var t_texture: texture_2d<f32>;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let light_dir = normalize(vec3<f32>(1.0, 1.0, 0.0));
    let kdiffuse = max(dot(in.world_normal, light_dir), 0.0);
    let kambient = 0.1; 
    let color = textureSample(t_texture, s_texture, vec2<f32>(in.uv.x, 1.0 - in.uv.y)); 
    return color * max(kdiffuse, kambient);
}