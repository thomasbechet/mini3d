struct CanvasData {
    resolution: vec2<u32>,
};
@group(0) @binding(0)
var<uniform> canvas: CanvasData;

struct BlitData {
    color: vec3<f32>,
    depth: f32,
    pos: u32,  // X (i16) | Y (i16)
    tex: u32,  // U (u16) | V (u16)
    size: u32, // W (u16) | H (u16)
};
@group(1) @binding(0)
var<uniform> blits: array<BlitData, 256>;

@group(1) @binding(1)
var t_texture: texture_2d<f32>;
@group(1) @binding(2)
var s_texture: sampler;

// Vertex Shader

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) @interpolate(flat) instance_index: u32,
};

@vertex
fn vs_main(
    @builtin(vertex_index) vertex_index: u32,
    @builtin(instance_index) instance_index: u32,
) -> VertexOutput {
    var out: VertexOutput;

    // Extract vertex data
    let pos = vec2<f32>(f32(blits[instance_index].pos & 0xFFFFu), f32(blits[instance_index].pos >> 16u));
    let tex = vec2<f32>(f32(blits[instance_index].tex & 0xFFFFu), f32(blits[instance_index].tex >> 16u));
    let size = vec2<f32>(f32(blits[instance_index].size & 0xFFFFu), f32(blits[instance_index].size >> 16u));
    let depth = blits[instance_index].depth;

    // Compute vertex offset based on the vertex index
    let offset = vec2<f32>(
        f32(((vertex_index & 2u) >> 1u) | ((vertex_index % 5u) >> 2u)),
        f32(vertex_index & 1u),
    );

    // Apply offset and normalize
    var position = (pos + size * offset) / vec2<f32>(canvas.resolution);
    position.y = 1.0 - position.y;
    let uv = (tex + size * offset) / vec2<f32>(textureDimensions(t_texture, 0));

    // Set output
    out.clip_position = vec4<f32>(position * 2.0 - 1.0, depth, 1.0);
    out.uv = uv;
    out.instance_index = instance_index;
    
    return out;
}

// Fragment shader

@fragment
fn fs_main(
    in: VertexOutput,
) -> @location(0) vec4<f32> {

    // Extract filtering color
    let filter_color = blits[in.instance_index].color;
    
    // Sample texture
    var color = textureSample(t_texture, s_texture, vec2<f32>(in.uv.x, in.uv.y));
    return color * vec4<f32>(filter_color, 1.0);
}