use crate::{
    feature::renderer::pass::GraphicsPassHandle,
    renderer::{
        command::{
            CanvasCommandBuffer, ComputeCommandBuffer, CopyCommandBuffer, GraphicsCommandBuffer,
        },
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

    /// Render graphs

    pub fn create_render_graph(ctx: &mut Context) {}

    pub fn destroy_render_graph(ctx: &mut Context, graph: RenderGraphHandle) {}

    pub fn set_render_graph(ctx: &mut Context, graph: RenderGraphHandle) {}

    /// Vertex buffers

    pub fn create_vertex_buffer(ctx: &mut Context) -> VertexBufferHandle {}

    pub fn destroy_vertex_buffer(ctx: &mut Context, buffer: VertexBufferHandle) {}

    pub fn update_vertex_buffer(ctx: &mut Context, buffer: VertexBufferHandle) {}

    /// Textures

    pub fn create_texture(ctx: &mut Context) -> TextureHandle {}

    pub fn destroy_texture(ctx: &mut Context, texture: TextureHandle) {}

    /// Font

    pub fn create_font(ctx: &mut Context) -> FontHandle {}

    pub fn destroy_font(ctx: &mut Context, font: FontHandle) {}

    /// Pipelines

    pub fn create_graphics_pipeline(ctx: &mut Context) -> GraphicsPipelineHandle {}

    pub fn destroy_graphics_pipeline(ctx: &mut Context, pipeline: GraphicsPipelineHandle) {}

    pub fn create_compute_pipeline(ctx: &mut Context) -> ComputePipelineHandle {}

    pub fn destroy_compute_pipeline(ctx: &mut Context, pipeline: ComputePipelineHandle) {}

    /// Graphics passes

    pub fn find_graphics_pass(ctx: &Context) -> GraphicsPassHandle {
        todo!()
    }

    pub fn begin_graphics_pass<'a>(ctx: &'a Context) -> GraphicsCommandBuffer<'a> {
        todo!()
    }

    pub fn end_graphics_pass(cmd: GraphicsCommandBuffer) {
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

    /// Copy passes

    pub fn create_copy_pass<'a>(ctx: &'a Context) -> CopyCommandBuffer<'a> {
        todo!()
    }

    pub fn end_copy_pass(cmd: CopyCommandBuffer) {
        todo!()
    }

    // Declaration
    // Invocation
    // Dispatch
}
