use glam::{Mat4, Vec4};

use crate::{
    feature::renderer::{
        buffer::BufferHandle,
        font::{Font, FontHandle},
        graph::{RenderGraph, RenderGraphError, RenderGraphHandle},
        mesh::{Mesh, MeshHandle},
        model::{Model, ModelHandle},
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

    pub fn find(ctx: &Context, key: impl ToUID) -> Option<TextureHandle> {
        ctx.resource.find_typed(key, ctx.renderer.handles.texture)
    }
}

impl Font {
    pub fn create(ctx: &mut Context) -> FontHandle {
        todo!()
    }

    pub fn destroy(ctx: &mut Context, font: FontHandle) {}

    pub fn find(ctx: &Context, key: impl ToUID) -> Option<FontHandle> {
        ctx.resource.find_typed(key, ctx.renderer.handles.font)
    }
}

impl Mesh {
    pub fn create(ctx: &mut Context) -> MeshHandle {
        todo!()
    }

    pub fn destroy(ctx: &mut Context, mesh: MeshHandle) {}

    pub fn find(ctx: &Context, key: impl ToUID) -> Option<MeshHandle> {
        ctx.resource.find_typed(key, ctx.renderer.handles.mesh)
    }
}

impl RenderGraph {
    pub fn create(ctx: &mut Context) -> RenderGraphHandle {
        todo!()
    }

    pub fn destroy(ctx: &mut Context, graph: RenderGraphHandle) {}

    pub fn find(ctx: &Context, key: impl ToUID) -> Option<RenderGraphHandle> {
        ctx.resource
            .find_typed(key, ctx.renderer.handles.render_graph)
    }
}

impl Model {
    pub fn create(ctx: &mut Context) -> ModelHandle {
        todo!()
    }

    pub fn destroy(ctx: &mut Context, model: ModelHandle) {}

    pub fn find(ctx: &Context, key: impl ToUID) -> Option<ModelHandle> {
        ctx.resource.find_typed(key, ctx.renderer.handles.model)
    }
}

pub struct Renderer;

impl Renderer {
    /// Statistics

    pub fn statistics(ctx: &Context) -> RendererStatistics {
        ctx.renderer.statistics()
    }

    /// Render graphs

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
