use crate::feature::renderer::buffer::{AttributeFormat, RenderBufferHandle};

use super::pipeline::{BlendMode, CullMode, GraphicsPipeline, GraphicsPipelineHandle};

type UniformHandle = u16;
enum UniformType {
    Buffer,
    Texture,
    TextureCube,
}
struct BindingEntry {
    ty: UniformType,
    handle: UniformHandle,
}

struct VertexAttributeEntry {
    format: AttributeFormat,
    buffer: RenderBufferHandle,
}

pub(crate) struct GraphicsCommandBuilder {
    vertex_attributes: [VertexAttributeEntry; GraphicsPipeline::MAX_VERTEX_ATTRIBUTE_COUNT],
    bindings: [BindingEntry; GraphicsPipeline::MAX_BINDINGS_COUNT],
    pipeline: GraphicsPipelineHandle,
    blend_mode: BlendMode,
    cull_mode: CullMode,
}
