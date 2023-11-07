use glam::{IVec2, Mat4, Vec2};

use crate::{
    api::Context,
    feature::renderer::{
        font::FontHandle,
        mesh::MeshHandle,
        pipeline::{ComputePipelineHandle, GraphicsPipelineHandle},
        texture::{TextureHandle, TextureWrapMode},
    },
    math::rect::IRect,
};

use super::color::Color;

pub struct GraphicsCommandBuffer {}

impl GraphicsCommandBuffer {
    pub fn set_pipeline(&mut self, ctx: &mut Context, pipeline: GraphicsPipelineHandle) {}

    pub fn set_vertex_buffer(&mut self, ctx: &mut Context, binding: u32) {}

    pub fn set_texture(&mut self, ctx: &mut Context, texture: TextureHandle, binding: u32) {}

    pub fn set_vec2(&mut self, ctx: &mut Context, binding: u32, value: Vec2) {}

    pub fn set_vec4(&mut self, ctx: &mut Context, binding: u32, value: Vec2) {}

    pub fn set_mat4(&mut self, ctx: &mut Context, binding: u32, value: Mat4) {}

    pub fn draw(&mut self, ctx: &mut Context, vertex_count: u32, instance_count: u32) {}

    pub fn draw_mesh(&mut self, ctx: &mut Context, mesh: MeshHandle) {}
}

pub struct CanvasCommandBuffer {}

impl CanvasCommandBuffer {
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

pub struct ComputeCommandBuffer {}

impl ComputeCommandBuffer {
    pub fn set_pipeline(&mut self, pipeline: ComputePipelineHandle) {}

    pub fn dispatch(&mut self, x: u32, y: u32, z: u32) {}
}

pub struct CopyCommandBuffer {}

impl CopyCommandBuffer {}
