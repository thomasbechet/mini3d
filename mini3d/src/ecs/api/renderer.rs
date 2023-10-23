use crate::renderer::{color::Color, graphics::Graphics, RendererStatistics};

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

    pub fn create_render_pass(ctx: &mut Context) {}

    pub fn begin_pass(ctx: &mut Context) {}

    pub fn end_pass(ctx: &mut Context) {}

    // Declaration
    // Invocation
    // Dispatch
}
