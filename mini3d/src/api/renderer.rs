use crate::{
    feature::renderer::{
        font::{Font, FontHandle},
        material::{Material, MaterialHandle},
        mesh::{Mesh, MeshHandle},
        model::{Model, ModelHandle},
        renderpass::{
            canvas::{CanvasPass, CanvasPassHandle},
            diffuse::{DiffusePass, DiffusePassHandle},
            RenderPass, RenderPassType,
        },
        texture::{Texture, TextureHandle, TextureWrapMode},
        transform::{RenderTransform, RenderTransformHandle},
    },
    math::{mat::M4I32F16, rect::IRect, vec::V2I32},
    renderer::{color::Color, RendererFeatures, RendererStatistics},
    resource::handle::ResourceHandle,
    utils::uid::ToUID,
};

use super::Context;

impl Texture {
    pub fn create(ctx: &mut Context) -> TextureHandle {
        todo!()
    }

    pub fn destroy(ctx: &mut Context, texture: TextureHandle) {}

    pub fn find(ctx: &Context, name: impl ToUID) -> Option<TextureHandle> {
        ctx.resource.find_typed(name, ctx.renderer.handles.texture)
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

    pub fn find(ctx: &Context, name: impl ToUID) -> Option<MeshHandle> {
        ctx.resource.find_typed(name, ctx.renderer.handles.mesh)
    }
}

impl Material {
    pub fn create(ctx: &mut Context) -> MaterialHandle {
        todo!()
    }

    pub fn destroy(ctx: &mut Context, material: MaterialHandle) {}

    pub fn find(ctx: &Context, name: impl ToUID) -> Option<MaterialHandle> {
        ctx.resource.find_typed(name, ctx.renderer.handles.material)
    }
}

impl Model {
    pub fn create(ctx: &mut Context) -> ModelHandle {
        todo!()
    }

    pub fn destroy(ctx: &mut Context, model: ModelHandle) {}

    pub fn find(ctx: &Context, name: impl ToUID) -> Option<ModelHandle> {
        ctx.resource.find_typed(name, ctx.renderer.handles.model)
    }
}

impl RenderTransform {
    pub fn create(ctx: &mut Context, interpolate: bool) -> RenderTransformHandle {
        todo!()
    }

    pub fn update(ctx: &mut Context, value: M4I32F16, teleport: bool) {
        todo!()
    }
}

pub struct Renderer;

impl Renderer {
    pub fn statistics(ctx: &Context) -> RendererStatistics {
        ctx.renderer.statistics()
    }

    pub fn features(ctx: &Context) -> RendererFeatures {
        ctx.renderer.features()
    }
}

impl DiffusePass {
    pub fn create(ctx: &mut Context, name: &str) -> DiffusePassHandle {
        todo!()
    }

    pub fn find(ctx: &Context, name: impl ToUID) -> Option<DiffusePassHandle> {
        let handle: ResourceHandle = ctx
            .resource
            .find_typed(name, ctx.renderer.handles.renderpass)
            .unwrap_or_default();
        let renderpass = ctx.resource.native_unchecked::<RenderPass>(handle);
        if matches!(renderpass.ty, RenderPassType::Diffuse) {
            Some(handle.into())
        } else {
            None
        }
    }

    pub fn render(ctx: &mut Context, pass: DiffusePassHandle) {
        todo!()
    }

    pub fn draw_mesh(
        ctx: &mut Context,
        pass: DiffusePassHandle,
        mesh: MeshHandle,
        material: MaterialHandle,
        transform: RenderTransformHandle,
        sort: u32,
    ) {
    }
}

impl CanvasPass {
    pub fn create(ctx: &mut Context, name: &str) -> CanvasPassHandle {
        todo!()
    }

    pub fn find(ctx: &Context, name: impl ToUID) -> Option<CanvasPassHandle> {
        let handle: ResourceHandle = ctx
            .resource
            .find_typed(name, ctx.renderer.handles.renderpass)
            .unwrap_or_default();
        let renderpass = ctx.resource.native_unchecked::<RenderPass>(handle);
        if matches!(renderpass.ty, RenderPassType::Canvas) {
            Some(handle.into())
        } else {
            None
        }
    }

    pub fn render(ctx: &mut Context, pass: CanvasPassHandle) {}

    pub fn set_scissor(ctx: &mut Context, extent: Option<IRect>) {}

    pub fn draw_rect(ctx: &mut Context, pass: CanvasPassHandle, extent: IRect, color: Color) {}

    pub fn draw_line(
        ctx: &mut Context,
        pass: CanvasPassHandle,
        x0: V2I32,
        x1: V2I32,
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
        texture: TextureHandle,
        extent: IRect,
        texture_extent: IRect,
        filtering: Color,
        wrap_mode: TextureWrapMode,
        alpha_threshold: u8,
    ) {
    }

    pub fn print(&mut self, pass: CanvasPassHandle, position: V2I32, text: &str, font: FontHandle) {
    }
}
