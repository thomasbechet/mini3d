use crate::{define_resource_handle, feature::renderer::buffer::AttributeFormat};

use super::uniform::UniformType;

define_resource_handle!(GraphicsPipelineHandle);
define_resource_handle!(ComputePipelineHandle);

pub enum PrimitiveTopology {
    Point,
    Line,
    Triangle,
}

pub enum BlendMode {
    Opaque,
    Alpha,
    Additive,
    Multiply,
}

pub enum CullMode {
    None,
    Front,
    Back,
}

pub struct VertexInputBinding {
    attribute: u16,
    format: AttributeFormat,
    location: u8,
}

pub struct VertexInputState {
    bindings: Vec<VertexInputBinding>,
}

pub struct UniformDescriptor {
    ty: UniformType,
}

pub struct GraphicsPipeline {
    pub vertex_input: VertexInputState,
    pub topology: PrimitiveTopology,
    pub blend_mode: BlendMode,
    pub cull_mode: CullMode,
    pub uniforms: [UniformType; Self::MAX_UNIFORM_COUNT],
    pub vertex_shader: u32,
    pub fragment_shader: u32,
}

impl GraphicsPipeline {
    pub const MAX_VERTEX_BUFFER_COUNT: usize = 8;
    pub const MAX_UNIFORM_COUNT: usize = 256;

    pub fn set_blend(&mut self, mode: BlendMode) {}
}

pub struct ComputePipeline;

impl ComputePipeline {}
