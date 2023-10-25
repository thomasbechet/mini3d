use super::{
    graphics_pipeline::GraphicsPipelineHandle, texture::TextureHandle,
    vertex_buffer::VertexBufferHandle,
};

pub struct RenderCommandBuffer<'a> {}

impl<'a> RenderCommandBuffer<'a> {
    pub fn bind_graphics_pipeline(&mut self, pipeline: GraphicsPipelineHandle) {}
    pub fn bind_vertex_buffer(&mut self, buffer: VertexBufferHandle) {}
    pub fn bind_texture(&mut self, texture: TextureHandle, binding: u32) {}
    pub fn draw(&mut self, vertex_count: u32, instance_count: u32) {}
}
