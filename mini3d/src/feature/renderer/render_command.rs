use super::{graphics_pipeline::GraphicsPipelineHandle, vertex_buffer::VertexBufferHandle};

pub struct RenderCommandBuffer;

pub struct RenderCommand;

impl RenderCommand {
    pub fn bind_graphics_pipeline(&mut self, pipeline: GraphicsPipelineHandle) {}
    pub fn bind_vertex_buffer(&mut self, buffer: VertexBufferHandle) {}
}
