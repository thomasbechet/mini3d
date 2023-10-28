use glam::{IVec2, Mat4, Vec2};

use crate::{
    feature::renderer::{
        buffer::BufferHandle,
        font::FontHandle,
        pipeline::{ComputePipelineHandle, GraphicsPipelineHandle},
        texture::TextureHandle,
    },
    math::rect::IRect,
};

use super::{color::Color, graphics::TextureWrapMode};

pub struct GraphicsCommandBuffer<'a> {}

impl<'a> GraphicsCommandBuffer<'a> {
    pub fn set_pipeline(&mut self, pipeline: GraphicsPipelineHandle) {}

    pub fn set_vertex_buffer(&mut self, buffer: BufferHandle) {}

    pub fn set_texture(&mut self, texture: TextureHandle, binding: u32) {}

    pub fn push_vec2(&mut self, binding: u32, value: Vec2) {}

    pub fn push_vec4(&mut self, binding: u32, value: Vec2) {}

    pub fn push_mat4(&mut self, binding: u32, value: Mat4) {}

    pub fn draw(&mut self, vertex_count: u32, instance_count: u32) {}
}

pub struct CanvasCommandBuffer<'a> {}

impl<'a> CanvasCommandBuffer<'a> {
    pub fn scissor(&mut self, extent: Option<IRect>) {}

    pub fn set_font(&mut self, font: FontHandle, binding: u32) {}

    pub fn draw_rect(&mut self, extent: IRect, color: Color) {}

    pub fn draw_line(&mut self, x0: IVec2, x1: IVec2, color: Color) {}

    pub fn draw_vline(&mut self, x: i32, y0: i32, y1: i32, color: Color) {}

    pub fn draw_hline(&mut self, y: i32, x0: i32, x1: i32, color: Color) {}

    pub fn fill_rect(&mut self, extent: IRect, color: Color) {}

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

    pub fn print(&mut self, position: IVec2, text: &str, font: FontHandle) {}
}

pub struct ComputeCommandBuffer<'a> {}

impl<'a> ComputeCommandBuffer<'a> {
    pub fn set_pipeline(&mut self, pipeline: ComputePipelineHandle) {}

    pub fn dispatch(&mut self, x: u32, y: u32, z: u32) {}
}

pub struct CopyCommandBuffer<'a> {}

impl<'a> CopyCommandBuffer<'a> {}
