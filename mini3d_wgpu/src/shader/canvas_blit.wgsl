struct CanvasData {
    resolution: vec2<u32>,
};
@group(0) @binding(0)
var<uniform> canvas: CanvasData;

@group(1) @binding(0)
var t_texture: texture_2d<f32>;
@group(1) @binding(1)
var s_texture: sampler;

// Vertex Shader

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) @interpolate(flat) filtering: vec3<f32>,
    @location(2) @interpolate(flat) threshold: f32,
};

@vertex
fn vs_main(
    @builtin(vertex_index) vertex_index: u32,
    @builtin(instance_index) instance_index: u32,
    @location(0) pos: u32,  // X (i16) | Y (i16)
    @location(1) tex: u32,  // U (u16) | V (u16)
    @location(2) size: u32, // W (u16) | H (u16)
    @location(3) depth: f32,
    @location(4) color: vec3<f32>,
    @location(5) threshold: f32,
) -> VertexOutput {
    var out: VertexOutput;

    // Extract vertex data
    let pos = vec2<f32>(f32(pos & 0xFFFFu), f32(pos >> 16u));
    let tex = vec2<f32>(f32(tex & 0xFFFFu), f32(tex >> 16u));
    let size = vec2<f32>(f32(size & 0xFFFFu), f32(size >> 16u));

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
    out.filtering = color;
    out.threshold = threshold;
    
    return out;
}

// Fragment shader

@fragment
fn fs_main(
    in: VertexOutput,
) -> @location(0) vec4<f32> {

    // Sample texture
    var color = textureSample(t_texture, s_texture, vec2<f32>(in.uv.x, in.uv.y));
    if color.a < in.threshold { discard; }
    return color * vec4<f32>(in.filtering, 1.0);
}