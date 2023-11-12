use glam::{IVec2, Mat4, Vec2, Vec3, Vec4};

use crate::{
    feature::renderer::{
        buffer::{Buffer, BufferHandle, BufferUsage},
        font::{Font, FontHandle},
        graph::{RenderGraph, RenderGraphError, RenderGraphHandle},
        mesh::{Mesh, MeshHandle},
        model::{Model, ModelHandle},
        pass::{CanvasPass, ComputePass, CopyPass, GraphicsPass},
        texture::{Texture, TextureHandle, TextureWrapMode},
    },
    math::rect::IRect,
    renderer::{
        color::Color,
        pipeline::{BlendMode, GraphicsPipelineHandle},
        queue::{CanvasQueueHandle, ComputeQueueHandle, CopyQueueHandle, GraphicsQueueHandle},
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

impl Buffer {
    pub fn create(ctx: &mut Context, usage: BufferUsage) -> BufferHandle {
        todo!()
    }

    pub fn find_attribute(ctx: &Context, buffer: BufferHandle, name: impl ToUID) -> Option<u8> {
        None
    }

    pub fn set_float(
        ctx: &mut Context,
        buffer: BufferHandle,
        index: u32,
        attribute: u8,
        value: f32,
    ) {
    }

    pub fn set_int(ctx: &mut Context, buffer: BufferHandle, index: u32, attribute: u8, value: i32) {
    }

    pub fn set_vec2(
        ctx: &mut Context,
        buffer: BufferHandle,
        index: u32,
        attribute: u8,
        value: Vec2,
    ) {
    }

    pub fn set_vec3(
        ctx: &mut Context,
        buffer: BufferHandle,
        index: u32,
        attribute: u8,
        value: Vec3,
    ) {
    }

    pub fn set_vec4(
        ctx: &mut Context,
        buffer: BufferHandle,
        index: u32,
        attribute: u8,
        value: Vec4,
    ) {
    }

    pub fn set_mat4(
        ctx: &mut Context,
        buffer: BufferHandle,
        index: u32,
        attribute: u8,
        value: Mat4,
    ) {
    }
}

pub struct Renderer;

impl Renderer {
    /// Statistics

    pub fn statistics(ctx: &Context) -> RendererStatistics {
        ctx.renderer.statistics()
    }

    /// Render graphs

    pub fn set_render_graph(
        ctx: &mut Context,
        graph: RenderGraphHandle,
    ) -> Result<(), RenderGraphError> {
        Ok(())
    }
}

impl GraphicsPass {
    pub fn queue(ctx: &Context, pass: impl ToUID) -> GraphicsQueueHandle {}
}

struct GraphicsCommand;

impl GraphicsCommand {
    pub fn set_vertex_buffer(ctx: &mut Context, queue: GraphicsQueueHandle, buffer: BufferHandle) {
        let queue = &mut ctx.renderer.graph.graphics_queues[queue.0 as usize];
        queue.draw.vertex_buffer = buffer;
    }

    pub fn set_pipeline(
        ctx: &mut Context,
        queue: GraphicsQueueHandle,
        pipeline: GraphicsPipelineHandle,
    ) {
        let queue = &mut ctx.renderer.graph.graphics_queues[queue.0 as usize];
        queue.draw.pipeline = pipeline;
    }

    pub fn set_viewport(ctx: &mut Context, queue: GraphicsQueueHandle, viewport: Vec4) {}

    pub fn set_scissor(ctx: &mut Context, queue: GraphicsQueueHandle, scissor: Vec4) {}

    pub fn set_blend_mode(ctx: &mut Context, queue: GraphicsQueueHandle, mode: BlendMode) {}

    pub fn set_cull_mode(ctx: &mut Context, queue: GraphicsQueueHandle, mode: BlendMode) {}

    pub fn bind_texture(
        ctx: &mut Context,
        queue: GraphicsQueueHandle,
        slot: u8,
        texture: TextureHandle,
    ) {
    }

    pub fn bind_buffer(
        ctx: &mut Context,
        queue: GraphicsQueueHandle,
        slot: u8,
        buffer: BufferHandle,
    ) {
    }

    pub fn push_int(ctx: &mut Context, queue: GraphicsQueueHandle, attribute: u8, value: i32) {}

    pub fn push_vec2(ctx: &mut Context, queue: GraphicsQueueHandle, attribute: u8, value: Vec2) {}

    pub fn push_vec3(ctx: &mut Context, queue: GraphicsQueueHandle, attribute: u8, value: Vec3) {}

    pub fn push_vec4(ctx: &mut Context, queue: GraphicsQueueHandle, attribute: u8, value: Vec4) {}

    pub fn push_mat4(ctx: &mut Context, queue: GraphicsQueueHandle, attribute: u8, value: Mat4) {}

    pub fn draw(ctx: &mut Context, queue: GraphicsQueueHandle, first: u32, count: u32, key: u32) {
        todo!()
    }

    pub fn draw_instanced(
        ctx: &mut Context,
        queue: GraphicsQueueHandle,
        first: u32,
        count: u32,
        instances: u32,
        key: u32,
    ) {
        todo!()
    }
}

impl CanvasPass {
    pub fn queue(ctx: &Context, pass: impl ToUID) -> CanvasQueueHandle {
        todo!()
    }
}

struct CanvasCommand;

impl CanvasCommand {
    pub fn set_scissor(ctx: &mut Context, queue: CanvasQueueHandle, extent: Option<IRect>) {}

    pub fn draw_rect(ctx: &mut Context, queue: CanvasQueueHandle, extent: IRect, color: Color) {}

    pub fn draw_line(
        ctx: &mut Context,
        queue: CanvasQueueHandle,
        x0: IVec2,
        x1: IVec2,
        color: Color,
    ) {
    }

    pub fn draw_vline(
        ctx: &mut Context,
        queue: CanvasQueueHandle,
        x: i32,
        y0: i32,
        y1: i32,
        color: Color,
    ) {
    }

    pub fn draw_hline(
        ctx: &mut Context,
        queue: CanvasQueueHandle,
        y: i32,
        x0: i32,
        x1: i32,
        color: Color,
    ) {
    }

    pub fn fill_rect(ctx: &mut Context, queue: CanvasQueueHandle, extent: IRect, color: Color) {}

    pub fn blit_texture(
        ctx: &mut Context,
        queue: CanvasQueueHandle,
        texture: TextureHandle,
        extent: IRect,
        texture_extent: IRect,
        filtering: Color,
        wrap_mode: TextureWrapMode,
        alpha_threshold: u8,
    ) {
    }

    pub fn print(&mut self, position: IVec2, text: &str, font: FontHandle) {}
}

impl ComputePass {
    pub fn queue(ctx: &Context, pass: impl ToUID) -> ComputeQueueHandle {
        todo!()
    }
}

struct ComputeCommand;

impl ComputeCommand {}

impl CopyPass {
    pub fn queue(ctx: &Context, pass: impl ToUID) -> CopyQueueHandle {
        todo!()
    }
}

struct CopyCommand;

impl CopyCommand {}
