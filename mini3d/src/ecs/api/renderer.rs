use crate::{
    feature::renderer::{
        render_graph::RenderGraphHandle, render_pass::RenderPassHandle, texture::TextureHandle,
        vertex_buffer::VertexBufferHandle,
    },
    renderer::{
        command::{CanvasCommandBuffer, ComputeCommandBuffer, RenderCommandBuffer},
        RendererStatistics,
    },
};

use super::Context;

pub struct Renderer;

impl Renderer {
    /// Statistics

    pub fn statistics(ctx: &Context) -> RendererStatistics {
        ctx.renderer.statistics()
    }

    /// Vertex buffers

    pub fn create_vertex_buffer(ctx: &mut Context) -> VertexBufferHandle {}

    pub fn destroy_vertex_buffer(ctx: &mut Context, buffer: VertexBufferHandle) {}

    pub fn update_vertex_buffer(ctx: &mut Context, buffer: VertexBufferHandle) {}

    /// Textures

    pub fn create_texture(ctx: &mut Context) -> TextureHandle {}

    pub fn destroy_texture(ctx: &mut Context, texture: TextureHandle) {}

    /// Pipelines

    pub fn create_graphics_pipeline(ctx: &mut Context) {}

    /// Render graphs

    pub fn create_render_graph(ctx: &mut Context) {}

    pub fn destroy_render_graph(ctx: &mut Context, graph: RenderGraphHandle) {}

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

    /// Canvas passes

    pub fn begin_canvas_pass<'a>(ctx: &'a Context) -> CanvasCommandBuffer<'a> {
        todo!()
    }

    pub fn end_canvas_pass(cmd: CanvasCommandBuffer) {
        todo!()
    }

    /// Compute passes

    pub fn create_compute_pass<'a>(ctx: &'a Context) -> ComputeCommandBuffer<'a> {
        todo!()
    }

    pub fn end_compute_pass(cmd: ComputeCommandBuffer) {
        todo!()
    }

    // Declaration
    // Invocation
    // Dispatch
}
