use glam::{Mat4, Vec2, Vec3, Vec4};

use crate::feature::renderer::{
    array::RenderArrayHandle,
    command::{CanvasCommandBufferHandle, GraphicsCommandBufferHandle, RenderCommandBufferHandle},
    constant::RenderConstantHandle,
    texture::TextureHandle,
};

use super::pipeline::{BlendMode, CullMode, GraphicsPipelineHandle};

type UniformHandle = u16;

struct VertexAttributeEntry {
    array: GPUArrayHandle,
}

struct PipelineState {
    pipeline: GraphicsPipelineHandle,
    blend_mode: BlendMode,
    cull_mode: CullMode,
}

struct VertexBufferParameter {
    buffer: u16,
    location: u8,
    offset: u16,
}

impl VertexBufferParameter {
    pub(crate) fn encode(&self) -> u32 {
        (self.buffer as u32) | ((self.location as u32) << 16) | ((self.offset as u32) << 24)
    }

    pub(crate) fn decode(w: u32) -> Self {
        Self {
            buffer: (w & 0xFFFF) as u16,
            location: ((w >> 16) & 0xFF) as u8,
            offset: ((w >> 24) & 0xFFFF) as u16,
        }
    }
}

struct UniformParameter {
    id: u16,
    slot: u8,
    ty: UniformType,
}

impl UniformParameter {
    pub(crate) fn encode(&self) -> u32 {
        (self.id as u32) | ((self.slot as u32) << 16) | ((self.ty as u32) << 24)
    }

    pub(crate) fn decode(w: u32) -> Self {
        Self {
            id: (w & 0xFFFF) as u16,
            slot: ((w >> 16) & 0xFF) as u8,
            ty: UniformType::Buffer,
        }
    }
}

pub(crate) struct GraphicsCommandEncoder {
    vertex_buffers: [VertexAttributeEntry; 8],
    uniforms: [GPUResourceHandle; 64],
    pso: PipelineState,
}

impl GraphicsCommandEncoder {
    pub fn set_pipeline(&mut self, pipeline: GraphicsPipelineHandle) {}

    pub fn set_viewport(&mut self, viewport: Vec4) {}

    pub fn set_scissor(&mut self, scissor: Vec4) {}

    pub fn set_blend_mode(&mut self, mode: BlendMode) {}

    pub fn set_cull_mode(&mut self, mode: CullMode) {}

    pub fn set_vertex_array(&mut self, array: RenderArrayHandle, location: u8) {}

    pub fn set_texture(&mut self, texture: TextureHandle, slot: u8) {}

    pub fn set_array(&mut self, array: RenderArrayHandle, slot: u8) {}

    pub fn set_constant(&mut self, constant: RenderConstantHandle, slot: u8) {}

    pub fn push_int(&mut self, slot: u8, value: i32) {}

    pub fn push_vec2(&mut self, slot: u8, value: Vec2) {}

    pub fn push_vec3(&mut self, slot: u8, value: Vec3) {}

    pub fn push_vec4(&mut self, slot: u8, value: Vec4) {}

    pub fn push_mat4(&mut self, slot: u8, value: Mat4) {}

    pub fn draw(&mut self, cmd: RenderCommandBufferHandle, first: u32, count: u32, key: u32) {
        todo!()
    }

    pub fn draw_instanced(
        &mut self,
        cmd: RenderCommandBufferHandle,
        first: u32,
        count: u32,
        instances: u32,
        key: u32,
    ) {
        todo!()
    }
}

pub(crate) struct ComputeCommandEncoder {}

pub(crate) struct CanvasCommandEncoder {}
