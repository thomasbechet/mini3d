use glam::{Mat4, Vec4};

use crate::{
    feature::renderer::{
        buffer::BufferHandle,
        font::{Font, FontHandle},
        graph::{RenderGraphError, RenderGraphHandle},
        pass::{
            CanvasPass, CanvasPassHandle, ComputePass, ComputePassHandle, CopyPass, CopyPassHandle,
            GraphicsPass, GraphicsPassHandle,
        },
        pipeline::{ComputePipelineHandle, GraphicsPipelineHandle},
        texture::{Texture, TextureHandle},
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

impl Texture {
    pub fn create(ctx: &mut Context) -> TextureHandle {
        todo!()
    }

    pub fn destroy(ctx: &mut Context, texture: TextureHandle) {}

    pub fn find(ctx: &Context, name: impl ToUID) -> TextureHandle {
        todo!()
    }
}

impl Font {
    pub fn create(ctx: &mut Context) -> FontHandle {
        todo!()
    }

    pub fn destroy(ctx: &mut Context, font: FontHandle) {}

    pub fn find(ctx: &Context, name: impl ToUID) -> FontHandle {
        todo!()
    }
}

pub struct Renderer;

impl Renderer {
    /// Statistics

    pub fn statistics(ctx: &Context) -> RendererStatistics {
        ctx.renderer.statistics()
    }

    /// Render graphs

    pub fn create_render_graph(ctx: &mut Context) -> RenderGraphHandle {
        Default::default()
    }

    pub fn destroy_render_graph(ctx: &mut Context, graph: RenderGraphHandle) {
        Default::default()
    }

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

    pub fn create_buffer(ctx: &mut Context) -> BufferHandle {
        Default::default()
    }

    pub fn destroy_buffer(ctx: &mut Context, buffer: BufferHandle) {}

    pub fn write_buffer_f32(ctx: &mut Context, buffer: BufferHandle, index: u32, data: f32) {}

    pub fn write_buffer_i32(ctx: &mut Context, buffer: BufferHandle, index: u32, data: i32) {}

    pub fn write_buffer_vec4(ctx: &mut Context, buffer: BufferHandle, index: u32, data: Vec4) {}

    pub fn write_buffer_mat4(ctx: &mut Context, buffer: BufferHandle, index: u32, data: Mat4) {}

    /// Pipelines

    pub fn create_graphics_pipeline(ctx: &mut Context) -> GraphicsPipelineHandle {
        Default::default()
    }

    pub fn destroy_graphics_pipeline(ctx: &mut Context, pipeline: GraphicsPipelineHandle) {}

    pub fn create_compute_pipeline(ctx: &mut Context) -> ComputePipelineHandle {
        Default::default()
    }

    pub fn destroy_compute_pipeline(ctx: &mut Context, pipeline: ComputePipelineHandle) {}
}

impl GraphicsPass {
    pub fn find(ctx: &Context, name: impl ToUID) -> GraphicsPassHandle {
        todo!()
    }

    pub fn begin(ctx: &Context, pass: GraphicsPassHandle) -> GraphicsCommandBuffer {
        todo!()
    }

    pub fn end(cmd: GraphicsCommandBuffer) {
        todo!()
    }
}

impl CanvasPass {
    pub fn find(ctx: &Context, name: impl ToUID) -> Option<CanvasPassHandle> {
        todo!()
    }

    pub fn begin(ctx: &Context, pass: CanvasPassHandle) -> CanvasCommandBuffer {
        todo!()
    }

    pub fn end(cmd: CanvasCommandBuffer) {
        todo!()
    }
}

impl ComputePass {
    pub fn find(ctx: &Context, name: impl ToUID) -> Option<ComputePassHandle> {
        todo!()
    }

    pub fn begin(ctx: &Context, pass: ComputePassHandle) -> ComputeCommandBuffer {
        todo!()
    }

    pub fn end(cmd: ComputeCommandBuffer) {
        todo!()
    }
}

impl CopyPass {
    pub fn find(ctx: &Context, name: impl ToUID) -> Option<CopyPassHandle> {
        todo!()
    }

    pub fn begin(ctx: &Context, pass: CopyPassHandle) -> CopyCommandBuffer {
        todo!()
    }

    pub fn end(cmd: CopyCommandBuffer) {
        todo!()
    }
}
