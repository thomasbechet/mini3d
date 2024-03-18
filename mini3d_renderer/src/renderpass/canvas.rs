use mini3d_math::{rect::IRect, vec::V2I32};

use crate::color::Color;

pub enum CanvasPassCommand {}

pub struct CanvasPassInfo {}

pub struct CanvasPassRenderInfo {}

pub struct CanvasPass {
}

impl CanvasPass {
    pub fn set_scissor(&mut self, extent: Option<IRect>) {}

    pub fn draw_rect(&mut self, extent: IRect, color: Color) {}

    pub fn draw_line(&mut self, x0: V2I32, x1: V2I32, color: Color) {}

    pub fn draw_vline(&mut self, x: i32, y0: i32, y1: i32, color: Color) {}

    pub fn draw_hline(&mut self, y: i32, x0: i32, x1: i32, color: Color) {}

    pub fn fill_rect(&mut self, extent: IRect, color: Color) {}

    // pub fn blit_texture(
    //     &mut self,
    //     texture: &Texture,
    //     extent: IRect,
    //     texture_extent: IRect,
    //     filtering: Color,
    //     wrap_mode: TextureWrapMode,
    //     alpha_threshold: u8,
    // ) {
    // }

    // pub fn draw_text(&mut self, position: V2I32, text: &str, font: &Font) {}
}
