use crate::resource::handle::ResourceHandle;

use super::{
    pipeline::{BlendMode, CullMode, GraphicsPipeline, GraphicsPipelineHandle},
    resource::{GPUArray, GPUArrayHandle, GPUFormat, GPUResourceHandle, ShaderResourceType},
    uniform::UniformType,
};

type UniformHandle = u16;

struct VertexAttributeEntry {
    array: GPUArrayHandle,
}

struct PipelineState {
    pipeline: GraphicsPipelineHandle,
    blend_mode: BlendMode,
    cull_mode: CullMode,
}

pub(crate) struct GraphicsCommandEncoder {
    vertex_buffers: [VertexAttributeEntry; 8],
    uniforms: [GPUResourceHandle; 64],
    pso: PipelineState,
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
