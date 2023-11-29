use glam::{IVec2, Mat4, Vec2, Vec3, Vec4};

use crate::{
    feature::renderer::{
        array::{RenderArray, RenderArrayHandle, RenderArrayUsage},
        font::{Font, FontHandle},
        material::MaterialHandle,
        mesh::{Mesh, MeshHandle},
        model::{Model, ModelHandle},
        renderpass::{
            CanvasPass, CanvasPassHandle, ForwardPass, ForwardPassHandle, RenderPass,
            RenderPassType,
        },
        texture::{GPUTexture, GPUTextureHandle, TextureWrapMode},
        variable::{RenderFormat, RenderVariable, RenderVariableHandle},
    },
    math::rect::IRect,
    renderer::{color::Color, RendererStatistics},
    resource::handle::ResourceHandle,
    utils::uid::ToUID,
};

use super::Context;

impl GPUTexture {
    pub fn create(ctx: &mut Context) -> GPUTextureHandle {
        todo!()
    }

    pub fn destroy(ctx: &mut Context, texture: GPUTextureHandle) {}

    pub fn find(ctx: &Context, key: impl ToUID) -> Option<GPUTextureHandle> {
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

impl Model {
    pub fn create(ctx: &mut Context) -> ModelHandle {
        todo!()
    }

    pub fn destroy(ctx: &mut Context, model: ModelHandle) {}

    pub fn find(ctx: &Context, key: impl ToUID) -> Option<ModelHandle> {
        ctx.resource.find_typed(key, ctx.renderer.handles.model)
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

impl RenderVariable {
    pub fn create(
        ctx: &mut Context,
        format: RenderFormat,
        interpolate: bool,
    ) -> RenderVariableHandle {
        todo!()
    }

    pub fn set_float(ctx: &mut Context, constant: RenderVariableHandle, value: f32) {}

    pub fn set_int(ctx: &mut Context, constant: RenderVariableHandle, value: i32) {}

    pub fn set_vec2(ctx: &mut Context, constant: RenderVariableHandle, value: Vec2) {}

    pub fn set_vec3(ctx: &mut Context, constant: RenderVariableHandle, value: Vec3) {}

    pub fn set_vec4(ctx: &mut Context, constant: RenderVariableHandle, value: Vec4) {}

    pub fn set_mat4(ctx: &mut Context, constant: RenderVariableHandle, value: Mat4) {}
}

pub struct Renderer;

impl Renderer {
    pub fn statistics(ctx: &Context) -> RendererStatistics {
        ctx.renderer.statistics()
    }
}

impl ForwardPass {
    pub fn create(ctx: &mut Context, name: &str) -> ForwardPassHandle {
        todo!()
    }

    pub fn find(ctx: &Context, key: impl ToUID) -> Option<ForwardPassHandle> {
        let handle: ResourceHandle = ctx
            .resource
            .find_typed(key, ctx.renderer.handles.renderpass)
            .unwrap_or_default();
        let renderpass = ctx.resource.native_unchecked::<RenderPass>(handle);
        if matches!(renderpass.ty, RenderPassType::Forward) {
            Some(handle.into())
        } else {
            None
        }
    }

    pub fn draw_model(
        ctx: &mut Context,
        pass: ForwardPassHandle,
        model: ModelHandle,
        material: MaterialHandle,
        transform: RenderVariableHandle,
        sort: u32,
    ) {
    }

    pub fn draw_mesh(
        ctx: &mut Context,
        pass: ForwardPassHandle,
        mesh: MeshHandle,
        material: MaterialHandle,
        transform: RenderVariableHandle,
        sort: u32,
    ) {
    }
}

impl CanvasPass {
    pub fn create(ctx: &mut Context, name: &str) -> CanvasPassHandle {
        todo!()
    }

    pub fn run(ctx: &mut Context, pass: CanvasPassHandle) {}

    pub fn set_scissor(ctx: &mut Context, extent: Option<IRect>) {}

    pub fn draw_rect(ctx: &mut Context, pass: CanvasPassHandle, extent: IRect, color: Color) {}

    pub fn draw_line(
        ctx: &mut Context,
        pass: CanvasPassHandle,
        x0: IVec2,
        x1: IVec2,
        color: Color,
    ) {
    }

    pub fn draw_vline(
        ctx: &mut Context,
        pass: CanvasPassHandle,
        x: i32,
        y0: i32,
        y1: i32,
        color: Color,
    ) {
    }

    pub fn draw_hline(
        ctx: &mut Context,
        pass: CanvasPassHandle,
        y: i32,
        x0: i32,
        x1: i32,
        color: Color,
    ) {
    }

    pub fn fill_rect(ctx: &mut Context, pass: CanvasPassHandle, extent: IRect, color: Color) {}

    pub fn blit_texture(
        ctx: &mut Context,
        pass: CanvasPassHandle,
        texture: GPUTextureHandle,
        extent: IRect,
        texture_extent: IRect,
        filtering: Color,
        wrap_mode: TextureWrapMode,
        alpha_threshold: u8,
    ) {
    }

    pub fn print(&mut self, pass: CanvasPassHandle, position: IVec2, text: &str, font: FontHandle) {
    }
}
