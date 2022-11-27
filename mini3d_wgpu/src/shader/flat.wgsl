// Vertex Shader

// group0 -> Global Bind Group
// group1 -> Mesh-Pass Bind Group
// group2 -> Flat-Material Bind Group

struct GlobalData {
    world_to_clip: mat4x4<f32>,
};
@group(0) @binding(0)
var<uniform> global: GlobalData;

struct ModelData {
    transform: mat4x4<f32>,
};
@group(0) @binding(1)
var<uniform> models: array<ModelData, 256>;

struct InstanceData {
    model_id: u32,
    batch_id: u32,
    _pad0: u32,
    _pad1: u32,
};
@group(1) @binding(0)
var<uniform> instances: array<InstanceData, 256>;

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
    @builtin(instance_index) instance_index: u32,
) -> VertexOutput {
    var out: VertexOutput;
    let model = models[instances[instance_index].model_id].transform;
    out.clip_position = global.world_to_clip * model * vec4<f32>(position, 1.0);
    out.world_normal = (model * vec4<f32>(normal, 0.0)).xyz;
    out.uv = uv;
    return out;
}

// Fragment shader

@group(0) @binding(2)
var s_texture: sampler;
@group(2) @binding(0)
var t_texture: texture_2d<f32>;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let light_dir = normalize(vec3<f32>(1.0, 1.0, 0.0));
    let kdiffuse = max(dot(in.world_normal, light_dir), 0.0);
    let kambient = 0.1;
    let color = textureSample(t_texture, s_texture, vec2<f32>(in.uv.x, 1.0 - in.uv.y)); 
    return color * max(kdiffuse, kambient);
}