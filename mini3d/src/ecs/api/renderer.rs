use glam::{Mat4, Vec4};

use crate::{
    feature::renderer::{
        buffer::BufferHandle,
        font::FontHandle,
        graph::{RenderGraphError, RenderGraphHandle},
        pass::{CanvasPassHandle, ComputePassHandle, CopyPassHandle, GraphicsPassHandle},
        pipeline::{ComputePipelineHandle, GraphicsPipelineHandle},
        texture::TextureHandle,
    },
    renderer::{
        command::{
            CanvasCommandBuffer, ComputeCommandBuffer, CopyCommandBuffer, GraphicsCommandBuffer,
        },
        RendererStatistics,
    },
    utils::uid::ToUID,
};

use super::Context;

pub struct Renderer;

impl Renderer {
    /// Statistics

    pub fn statistics(ctx: &Context) -> RendererStatistics {
        ctx.renderer.statistics()
    }

    /// Render graphs

    pub fn create_render_graph(ctx: &mut Context) -> RenderGraphHandle {}

    pub fn destroy_render_graph(ctx: &mut Context, graph: RenderGraphHandle) {}

    pub fn compile_render_graph(
        ctx: &mut Context,
        graph: RenderGraphHandle,
    ) -> Result<(), RenderGraphError> {
        Ok(())
    }

    pub fn use_render_graph(
        ctx: &mut Context,
        graph: RenderGraphHandle,
    ) -> Result<(), RenderGraphError> {
        Ok(())
    }

    /// Buffers

    pub fn create_buffer(ctx: &mut Context) -> BufferHandle {}

    pub fn destroy_buffer(ctx: &mut Context, buffer: BufferHandle) {}

    pub fn write_buffer_f32(ctx: &mut Context, buffer: BufferHandle, index: u32, data: f32) {}

    pub fn write_buffer_i32(ctx: &mut Context, buffer: BufferHandle, index: u32, data: i32) {}

    pub fn write_buffer_vec4(ctx: &mut Context, buffer: BufferHandle, index: u32, data: Vec4) {}

    pub fn write_buffer_mat4(ctx: &mut Context, buffer: BufferHandle, index: u32, data: Mat4) {}

    /// Textures

    pub fn create_texture(ctx: &mut Context) -> TextureHandle {}

    pub fn destroy_texture(ctx: &mut Context, texture: TextureHandle) {}

    pub fn update_texture(ctx: &mut Context, texture: TextureHandle) {}

    /// Font

    pub fn create_font(ctx: &mut Context) -> FontHandle {}

    pub fn destroy_font(ctx: &mut Context, font: FontHandle) {}

    /// Pipelines

    pub fn create_graphics_pipeline(ctx: &mut Context) -> GraphicsPipelineHandle {}

    pub fn destroy_graphics_pipeline(ctx: &mut Context, pipeline: GraphicsPipelineHandle) {}

    pub fn create_compute_pipeline(ctx: &mut Context) -> ComputePipelineHandle {}

    pub fn destroy_compute_pipeline(ctx: &mut Context, pipeline: ComputePipelineHandle) {}

    /// Graphics passes

    pub fn find_graphics_pass(ctx: &Context, name: impl ToUID) -> GraphicsPassHandle {
        todo!()
    }

    pub fn begin_graphics_pass<'a>(
        ctx: &'a Context,
        pass: GraphicsPassHandle,
    ) -> GraphicsCommandBuffer<'a> {
        todo!()
    }

    pub fn end_graphics_pass(cmd: GraphicsCommandBuffer) {
        todo!()
    }

    /// Canvas passes

    pub fn find_canvas_pass(ctx: &Context, name: impl ToUID) -> Option<CanvasPassHandle> {
        todo!()
    }

    pub fn begin_canvas_pass<'a>(
        ctx: &'a Context,
        pass: CanvasPassHandle,
    ) -> CanvasCommandBuffer<'a> {
        todo!()
    }

    pub fn end_canvas_pass(cmd: CanvasCommandBuffer) {
        todo!()
    }

    /// Compute passes

    pub fn find_compute_pass(ctx: &Context, name: impl ToUID) -> Option<ComputePassHandle> {
        todo!()
    }

    pub fn begin_compute_pass<'a>(
        ctx: &'a Context,
        pass: ComputePassHandle,
    ) -> ComputeCommandBuffer<'a> {
        todo!()
    }

    pub fn end_compute_pass(cmd: ComputeCommandBuffer) {
        todo!()
    }

    /// Copy passes

    pub fn find_copy_pass(ctx: &Context, name: impl ToUID) -> Option<CopyPassHandle> {
        todo!()
    }

    pub fn begin_copy_pass<'a>(ctx: &'a Context, pass: CopyPassHandle) -> CopyCommandBuffer<'a> {
        todo!()
    }

    pub fn end_copy_pass(cmd: CopyCommandBuffer) {
        todo!()
    }
}
