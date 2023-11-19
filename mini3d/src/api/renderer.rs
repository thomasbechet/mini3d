use glam::{IVec2, Mat4, Vec2, Vec3, Vec4};

use crate::{
    feature::renderer::{
        array::{RenderArray, RenderArrayHandle, RenderArrayUsage, RenderFormat},
        command::{RenderCommandBuffer, RenderCommandBufferHandle, RenderCommandBufferType},
        constant::{RenderConstant, RenderConstantHandle},
        font::{Font, FontHandle},
        graph::{RenderGraph, RenderGraphError, RenderGraphHandle, RenderTarget},
        mesh::{Mesh, MeshHandle},
        model::{Model, ModelHandle},
        pipeline::GraphicsPipelineHandle,
        texture::{Texture, TextureHandle, TextureWrapMode},
    },
    math::rect::IRect,
    renderer::{color::Color, RendererStatistics},
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
        ctx.resource.find_typed(key, ctx.renderer.handles.graph)
    }

    pub fn run(
        ctx: &mut Context,
        graph: RenderGraphHandle,
        target: RenderTarget,
        buffers: &[RenderCommandBufferHandle],
    ) -> Result<(), RenderGraphError> {
        todo!()
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

impl RenderCommandBuffer {
    pub fn create(ctx: &mut Context, ty: RenderCommandBufferType) -> RenderCommandBufferHandle {
        todo!()
    }

    pub fn find(ctx: &Context, key: impl ToUID) -> Option<RenderCommandBufferHandle> {
        ctx.resource
            .find_typed(key, ctx.renderer.handles.command_buffer)
    }
}

impl RenderArray {
    pub fn create(
        ctx: &mut Context,
        format: RenderFormat,
        size: u32,
        usage: RenderArrayUsage,
    ) -> RenderArrayHandle {
        todo!()
    }

    pub fn set_float(ctx: &mut Context, array: RenderArrayHandle, index: u32, value: f32) {}

    pub fn set_int(ctx: &mut Context, array: RenderArrayHandle, index: u32, value: i32) {}

    pub fn set_vec2(ctx: &mut Context, array: RenderArrayHandle, index: u32, value: Vec2) {}

    pub fn set_vec3(ctx: &mut Context, array: RenderArrayHandle, index: u32, value: Vec3) {}

    pub fn set_vec4(ctx: &mut Context, array: RenderArrayHandle, index: u32, value: Vec4) {}

    pub fn set_mat4(ctx: &mut Context, array: RenderArrayHandle, index: u32, value: Mat4) {}
}

impl RenderConstant {
    pub fn create(ctx: &mut Context, format: RenderFormat) -> RenderConstantHandle {
        todo!()
    }

    pub fn set_float(ctx: &mut Context, constant: RenderConstantHandle, value: f32) {}

    pub fn set_int(ctx: &mut Context, constant: RenderConstantHandle, value: i32) {}

    pub fn set_vec2(ctx: &mut Context, constant: RenderConstantHandle, value: Vec2) {}

    pub fn set_vec3(ctx: &mut Context, constant: RenderConstantHandle, value: Vec3) {}

    pub fn set_vec4(ctx: &mut Context, constant: RenderConstantHandle, value: Vec4) {}

    pub fn set_mat4(ctx: &mut Context, constant: RenderConstantHandle, value: Mat4) {}
}

pub struct Renderer;

impl Renderer {
    pub fn statistics(ctx: &Context) -> RendererStatistics {
        ctx.renderer.statistics()
    }
}

struct GraphicsCommand;

impl GraphicsCommand {
    pub fn set_pipeline(ctx: &mut Context, pipeline: GraphicsPipelineHandle) {
        ctx.renderer.graphics_encoder.set_pipeline(pipeline);
    }

    pub fn set_viewport(ctx: &mut Context, viewport: Vec4) {}

    pub fn set_scissor(ctx: &mut Context, scissor: Vec4) {}

    pub fn set_blend_mode(ctx: &mut Context, mode: BlendMode) {}

    pub fn set_cull_mode(ctx: &mut Context, mode: BlendMode) {}

    pub fn set_vertex_array(ctx: &mut Context, array: RenderArrayHandle, location: u8) {}

    pub fn set_texture(ctx: &mut Context, texture: TextureHandle, slot: u8) {}

    pub fn set_array(ctx: &mut Context, array: RenderArrayHandle, slot: u8) {}

    pub fn set_constant(ctx: &mut Context, constant: RenderConstantHandle, slot: u8) {}

    pub fn push_int(ctx: &mut Context, slot: u8, value: i32) {}

    pub fn push_vec2(ctx: &mut Context, slot: u8, value: Vec2) {}

    pub fn push_vec3(ctx: &mut Context, slot: u8, value: Vec3) {}

    pub fn push_vec4(ctx: &mut Context, slot: u8, value: Vec4) {}

    pub fn push_mat4(ctx: &mut Context, slot: u8, value: Mat4) {}

    pub fn draw(
        ctx: &mut Context,
        cmd: RenderCommandBufferHandle,
        first: u32,
        count: u32,
        key: u32,
    ) {
        todo!()
    }

    pub fn draw_instanced(
        ctx: &mut Context,
        cmd: RenderCommandBufferHandle,
        first: u32,
        count: u32,
        instances: u32,
        key: u32,
    ) {
        todo!()
    }
}

struct CanvasCommand;

impl CanvasCommand {
    pub fn set_scissor(ctx: &mut Context, extent: Option<IRect>) {}

    pub fn draw_rect(
        ctx: &mut Context,
        cmd: RenderCommandBufferHandle,
        extent: IRect,
        color: Color,
    ) {
    }

    pub fn draw_line(
        ctx: &mut Context,
        cmd: RenderCommandBufferHandle,
        x0: IVec2,
        x1: IVec2,
        color: Color,
    ) {
    }

    pub fn draw_vline(
        ctx: &mut Context,
        cmd: RenderCommandBufferHandle,
        x: i32,
        y0: i32,
        y1: i32,
        color: Color,
    ) {
    }

    pub fn draw_hline(
        ctx: &mut Context,
        cmd: RenderCommandBufferHandle,
        y: i32,
        x0: i32,
        x1: i32,
        color: Color,
    ) {
    }

    pub fn fill_rect(
        ctx: &mut Context,
        cmd: RenderCommandBufferHandle,
        extent: IRect,
        color: Color,
    ) {
    }

    pub fn blit_texture(
        ctx: &mut Context,
        cmd: RenderCommandBufferHandle,
        texture: TextureHandle,
        extent: IRect,
        texture_extent: IRect,
        filtering: Color,
        wrap_mode: TextureWrapMode,
        alpha_threshold: u8,
    ) {
    }

    pub fn print(
        &mut self,
        cmd: RenderCommandBufferHandle,
        position: IVec2,
        text: &str,
        font: FontHandle,
    ) {
    }
}

struct ComputeCommand;

impl ComputeCommand {}
