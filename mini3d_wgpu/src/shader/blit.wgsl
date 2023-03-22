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

fn uv_filtering(uv: vec2<f32>, texture_size: vec2<i32>) -> vec2<f32> {
    let pixel = uv * vec2<f32>(texture_size);

    let seam = floor(pixel + 0.5);
    let dudv = fwidth(pixel);
    let rel = (pixel - seam) / dudv;
    let mid_pix = vec2<f32>(0.5, 0.5);
    let pixel = seam + clamp(rel, -mid_pix, mid_pix);
    
    return pixel / vec2<f32>(texture_size);
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let uv = vec2<f32>(in.uv.x, 1.0 - in.uv.y);
    let uv = uv_filtering(uv, textureDimensions(t_texture));
    return textureSample(t_texture, s_texture, uv);
}