use glam::IVec2;

use crate::{
    feature::renderer::{
        compute_pipeline::ComputePipelineHandle, font::FontHandle,
        render_pipeline::RenderPipelineHandle, texture::TextureHandle,
        vertex_buffer::VertexBufferHandle,
    },
    math::rect::IRect,
};

use super::{color::Color, graphics::TextureWrapMode};

pub struct RenderCommandBuffer<'a> {}

impl<'a> RenderCommandBuffer<'a> {
    pub fn set_pipeline(&mut self, pipeline: RenderPipelineHandle) {}
    pub fn set_vertex_buffer(&mut self, buffer: VertexBufferHandle) {}
    pub fn set_texture(&mut self, texture: TextureHandle, binding: u32) {}
    pub fn draw(&mut self, vertex_count: u32, instance_count: u32) {}
}

pub struct CanvasCommandBuffer<'a> {}

impl<'a> CanvasCommandBuffer<'a> {
    pub fn print(&mut self, position: IVec2, text: &str, font: FontHandle) {}

    pub fn blit_texture(
        &mut self,
        texture: TextureHandle,
        extent: IRect,
        texture_extent: IRect,
        filtering: Color,
        wrap_mode: TextureWrapMode,
        alpha_threshold: u8,
    ) {
    }

    pub fn fill_rect(&mut self, extent: IRect, color: Color) {}

    pub fn draw_rect(&mut self, extent: IRect, color: Color) {}

    pub fn draw_line(&mut self, x0: IVec2, x1: IVec2, color: Color) {}

    pub fn draw_vline(&mut self, x: i32, y0: i32, y1: i32, color: Color) {}

    pub fn draw_hline(&mut self, y: i32, x0: i32, x1: i32, color: Color) {}

    pub fn scissor(&mut self, extent: Option<IRect>) {}
}

pub struct ComputeCommandBuffer<'a> {}

impl<'a> ComputeCommandBuffer<'a> {
    pub fn set_pipeline(&mut self, pipeline: ComputePipelineHandle) {}

    pub fn dispatch(&mut self, x: u32, y: u32, z: u32) {}
}
