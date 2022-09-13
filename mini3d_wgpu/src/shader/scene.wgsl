// Vertex Shader

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_normal: vec3<f32>,
    @location(1) uv: vec2<f32>,
};

struct SceneUniform {
    local_to_clip: mat4x4<f32>,
    local_to_world: mat4x4<f32>,
};
@group(0) @binding(0)
var<uniform> scene: SceneUniform;

@vertex
fn vs_main(
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = scene.local_to_clip * vec4<f32>(position, 1.0);
    out.world_normal = (scene.local_to_world * vec4<f32>(normal, 0.0)).xyz;
    out.uv = uv;
    return out;
}

// Fragment shader

@group(0) @binding(1)
var t_texture: texture_2d<f32>;
@group(0) @binding(2)
var s_texture: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let light_dir = normalize(vec3<f32>(1.0, 1.0, 0.0));
    let kdiffuse = max(dot(in.world_normal, light_dir), 0.0);
    let kambient = 0.1; 
    let color = textureSample(t_texture, s_texture, vec2<f32>(in.uv.x, 1.0 - in.uv.y)); 
    return color * max(kdiffuse, kambient);
}