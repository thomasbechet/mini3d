use crate::define_resource_handle;

use super::{array::RenderFormat, texture::TextureFormat};

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

pub enum VertexInputRate {
    PerVertex,
    PerInstance,
}

pub struct VertexInput {
    format: RenderFormat,
    rate: VertexInputRate,
}

pub enum PipelineResource {
    Array { format: RenderFormat, size: u32 },
    Constant { format: RenderFormat },
    Texture { format: TextureFormat },
}

pub struct PipelineResourceLayout {}

pub struct PushConstant {
    format: RenderFormat,
}

pub struct GraphicsPipelineLayout {
    vertex_inputs: [VertexInput; GraphicsPipeline::MAX_VERTEX_INPUT_COUNT],
    vertex_input_count: u8,
    resources: [PipelineResource; GraphicsPipeline::MAX_RESOURCE_COUNT],
    resource_count: u8,
    push_constants: [PushConstant; GraphicsPipeline::MAX_PUSH_CONSTANT_SIZE],
    push_constant_count: u8,
}

pub struct GraphicsPipeline {
    pub layout: GraphicsPipelineLayout,
    pub topology: PrimitiveTopology,
    pub blend_mode: BlendMode,
    pub cull_mode: CullMode,
    pub vertex_shader: u32,
    pub fragment_shader: u32,
}

impl GraphicsPipeline {
    pub const NAME: &'static str = "RTY_GraphicsPipeline";
    pub const MAX_VERTEX_INPUT_COUNT: usize = 8;
    pub const MAX_RESOURCE_COUNT: usize = 256;
    pub const MAX_PUSH_CONSTANT_SIZE: usize = 8;
}

define_resource_handle!(GraphicsPipelineHandle);

pub struct ComputePipeline;

impl ComputePipeline {
    pub const NAME: &'static str = "RTY_ComputePipeline";
}

define_resource_handle!(ComputePipelineHandle);
