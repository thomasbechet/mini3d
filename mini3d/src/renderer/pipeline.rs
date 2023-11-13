use crate::{define_resource_handle, feature::renderer::buffer::AttributeFormat};

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

pub enum UniformType {}

pub struct UniformBinding {
    location: u8,
    ty: AttributeFormat,
}

pub struct GraphicsPipeline {
    pub vertex_input: VertexInputState,
    pub topology: PrimitiveTopology,
    pub blend_mode: BlendMode,
    pub cull_mode: CullMode,
    pub uniforms: Vec<UniformBinding>,
    pub vertex_shader: u32,
    pub fragment_shader: u32,
}

impl GraphicsPipeline {
    pub const MAX_VERTEX_ATTRIBUTE_COUNT: usize = 8;
    pub const MAX_BINDINGS_COUNT: usize = 16;

    pub fn set_blend(&mut self, mode: BlendMode) {}
}

pub struct ComputePipeline;

impl ComputePipeline {}
