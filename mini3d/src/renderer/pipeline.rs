use crate::define_resource_handle;

use super::resource::{GPUFormat, GPUTextureFormat};

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

pub enum VertexInputRate {
    PerVertex,
    PerInstance,
}

pub struct VertexInputBinding {
    format: GPUFormat,
    rate: VertexInputRate,
}

pub enum ResourceBinding {
    Array { format: GPUFormat, size: u32 },
    Constant { format: GPUFormat },
    Texture { format: GPUTextureFormat },
}

pub struct PushConstantBinding {
    format: GPUFormat,
}

pub struct GraphicsPipelineLayout {
    vertex_inputs: [VertexInputBinding; GraphicsPipeline::MAX_VERTEX_INPUT_COUNT],
    vertex_input_count: u8,
    resources: [ResourceBinding; GraphicsPipeline::MAX_RESOURCE_COUNT],
    resource_count: u8,
    push_constants: [PushConstantBinding; GraphicsPipeline::MAX_PUSH_CONSTANT_SIZE],
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
    pub const MAX_VERTEX_INPUT_COUNT: usize = 8;
    pub const MAX_RESOURCE_COUNT: usize = 256;
    pub const MAX_PUSH_CONSTANT_SIZE: usize = 8;

    pub fn set_blend(&mut self, mode: BlendMode) {}
}

pub struct ComputePipeline;

impl ComputePipeline {}
