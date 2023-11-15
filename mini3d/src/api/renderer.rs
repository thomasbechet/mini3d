use glam::{IVec2, Mat4, Vec2, Vec3, Vec4};

use crate::{
    feature::renderer::{
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
        resource::{
            GPUArray, GPUArrayHandle, GPUArrayUsage, GPUConstant, GPUConstantHandle, GPUFormat,
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

impl GPUArray {
    pub fn create(
        ctx: &mut Context,
        format: GPUFormat,
        size: u32,
        usage: GPUArrayUsage,
    ) -> GPUArrayHandle {
        todo!()
    }

    pub fn set_float(ctx: &mut Context, array: GPUArrayHandle, index: u32, value: f32) {}

    pub fn set_int(ctx: &mut Context, array: GPUArrayHandle, index: u32, value: i32) {}

    pub fn set_vec2(ctx: &mut Context, array: GPUArrayHandle, index: u32, value: Vec2) {}

    pub fn set_vec3(ctx: &mut Context, array: GPUArrayHandle, index: u32, value: Vec3) {}

    pub fn set_vec4(ctx: &mut Context, array: GPUArrayHandle, index: u32, value: Vec4) {}

    pub fn set_mat4(ctx: &mut Context, array: GPUArrayHandle, index: u32, value: Mat4) {}
}

impl GPUConstant {
    pub fn create(ctx: &mut Context, format: GPUFormat) -> GPUConstantHandle {
        todo!()
    }

    pub fn set_float(ctx: &mut Context, constant: GPUConstantHandle, value: f32) {}

    pub fn set_int(ctx: &mut Context, constant: GPUConstantHandle, value: i32) {}

    pub fn set_vec2(ctx: &mut Context, constant: GPUConstantHandle, value: Vec2) {}

    pub fn set_vec3(ctx: &mut Context, constant: GPUConstantHandle, value: Vec3) {}

    pub fn set_vec4(ctx: &mut Context, constant: GPUConstantHandle, value: Vec4) {}

    pub fn set_mat4(ctx: &mut Context, constant: GPUConstantHandle, value: Mat4) {}
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

struct GraphicsCmd;

impl GraphicsCmd {
    pub fn set_pipeline(ctx: &mut Context, pipeline: GraphicsPipelineHandle) {}

    pub fn set_viewport(ctx: &mut Context, viewport: Vec4) {}

    pub fn set_scissor(ctx: &mut Context, scissor: Vec4) {}

    pub fn set_blend_mode(ctx: &mut Context, mode: BlendMode) {}

    pub fn set_cull_mode(ctx: &mut Context, mode: BlendMode) {}

    pub fn set_vertex_array(ctx: &mut Context, array: GPUArrayHandle, location: u8) {}

    pub fn set_texture(ctx: &mut Context, texture: TextureHandle, slot: u8) {}

    pub fn set_array(ctx: &mut Context, array: GPUArrayHandle, slot: u8) {}

    pub fn set_constant(ctx: &mut Context, constant: GPUConstantHandle, slot: u8) {}

    pub fn push_int(ctx: &mut Context, slot: u8, value: i32) {}

    pub fn push_vec2(ctx: &mut Context, slot: u8, value: Vec2) {}

    pub fn push_vec3(ctx: &mut Context, slot: u8, value: Vec3) {}

    pub fn push_vec4(ctx: &mut Context, slot: u8, value: Vec4) {}

    pub fn push_mat4(ctx: &mut Context, slot: u8, value: Mat4) {}

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

struct CanvasCmd;

impl CanvasCmd {
    pub fn set_scissor(ctx: &mut Context, extent: Option<IRect>) {}

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

struct ComputeCmd;

impl ComputeCmd {}

impl CopyPass {
    pub fn queue(ctx: &Context, pass: impl ToUID) -> CopyQueueHandle {
        todo!()
    }
}

struct CopyCmd;

impl CopyCmd {}
