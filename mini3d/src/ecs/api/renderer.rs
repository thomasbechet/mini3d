use crate::{
    feature::renderer::{
        render_command::RenderCommandBuffer, render_graph::RenderGraphHandle,
        render_pass::RenderPassHandle,
    },
    renderer::{color::Color, graphics::Graphics, RendererStatistics},
};

use super::Context;

pub struct Renderer;

impl Renderer {
    pub fn graphics<'a>(ctx: &'a mut Context) -> &'a mut Graphics {
        ctx.renderer.graphics()
    }

    pub fn set_clear_color(ctx: &mut Context, color: Color) {
        ctx.renderer.set_clear_color(color);
    }

    pub fn statistics(ctx: &Context) -> RendererStatistics {
        ctx.renderer.statistics()
    }

    pub fn create_vertex_buffer(ctx: &mut Context) {}

    pub fn create_texture(ctx: &mut Context) {}

    pub fn create_graphics_pipeline(ctx: &mut Context) {}

    pub fn create_render_graph(ctx: &mut Context) {}

    pub fn set_render_graph(ctx: &mut Context, graph: RenderGraphHandle) {}

    pub fn find_render_pass(ctx: &Context) -> RenderPassHandle {
        todo!()
    }

    pub fn begin_render_pass(ctx: &Context) -> RenderCommandBuffer {
        todo!()
    }

    pub fn end_render_pass(ctx: &Context, cmd: RenderCommandBuffer) {
        todo!()
    }

    // Declaration
    // Invocation
    // Dispatch
}
