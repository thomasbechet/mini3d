use mini3d_derive::Reflect;
use mini3d_derive::Serialize;

use crate::ecs::context::Context;
use crate::math::rect::IRect;
use crate::math::vec::V2I32;
use crate::renderer::color::Color;
use crate::renderer::component::Font;
use crate::renderer::component::Texture;
use crate::renderer::component::TextureWrapMode;
use crate::renderer::provider::RendererProviderHandle;

pub enum CanvasPassCommand {}

pub struct CanvasPassInfo {}

pub struct CanvasPassRenderInfo {}

#[derive(Default, Reflect, Serialize)]
pub struct CanvasPass {
    #[serialize(skip)]
    pub(crate) handle: RendererProviderHandle,
}

impl CanvasPass {
    pub fn render(&mut self, ctx: &mut Context) {}

    pub fn set_scissor(&mut self, ctx: &mut Context, extent: Option<IRect>) {}

    pub fn draw_rect(&mut self, ctx: &mut Context, extent: IRect, color: Color) {}

    pub fn draw_line(&mut self, ctx: &mut Context, x0: V2I32, x1: V2I32, color: Color) {}

    pub fn draw_vline(&mut self, ctx: &mut Context, x: i32, y0: i32, y1: i32, color: Color) {}

    pub fn draw_hline(&mut self, ctx: &mut Context, y: i32, x0: i32, x1: i32, color: Color) {}

    pub fn fill_rect(&mut self, ctx: &mut Context, extent: IRect, color: Color) {}

    pub fn blit_texture(
        &mut self,
        ctx: &mut Context,
        texture: &Texture,
        extent: IRect,
        texture_extent: IRect,
        filtering: Color,
        wrap_mode: TextureWrapMode,
        alpha_threshold: u8,
    ) {
    }

    pub fn print(&mut self, ctx: &mut Context, position: V2I32, text: &str, font: &Font) {}
}
