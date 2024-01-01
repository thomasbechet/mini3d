use crate::renderer::{RendererFeatures, RendererStatistics};

use super::Context;

pub struct Renderer;

impl Renderer {
    pub fn statistics(ctx: &Context) -> RendererStatistics {
        ctx.renderer.statistics()
    }

    pub fn features(ctx: &Context) -> RendererFeatures {
        ctx.renderer.features()
    }
}
