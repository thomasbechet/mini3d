use crate::{
    feature::renderer::{
        render_command::RenderCommandBuffer, render_graph::RenderGraphHandle,
        render_pass::RenderPassHandle, vertex_buffer::VertexBufferHandle,
    },
    renderer::{color::Color, graphics::Graphics, RendererStatistics},
    utils::uid::ToUID,
};

use super::Context;

pub struct Renderer;

impl Renderer {
    pub fn statistics(ctx: &Context) -> RendererStatistics {
        ctx.renderer.statistics()
    }

    /// Vertex buffers

    pub fn create_vertex_buffer(ctx: &mut Context) -> VertexBufferHandle {}

    pub fn destroy_vertex_buffer(ctx: &mut Context, buffer: VertexBufferHandle) {}

    /// Textures

    pub fn create_texture(ctx: &mut Context) -> TextureH {}

    /// Pipelines

    pub fn create_graphics_pipeline(ctx: &mut Context) {}

    /// Render graphs

    pub fn create_render_graph(ctx: &mut Context) {}

    pub fn set_render_graph(ctx: &mut Context, graph: RenderGraphHandle) {}

    /// Render passes

    pub fn find_render_pass(ctx: &Context) -> RenderPassHandle {
        todo!()
    }

    pub fn begin_render_pass<'a>(ctx: &'a Context) -> RenderCommandBuffer<'a> {
        todo!()
    }

    pub fn end_render_pass(cmd: RenderCommandBuffer) {
        todo!()
    }

    // Declaration
    // Invocation
    // Dispatch
}
