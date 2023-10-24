use crate::resource::handle::ResourceHandle;

use super::render_pass::RenderPassHandle;

pub struct RenderGraphHandle(pub(crate) ResourceHandle);

pub struct RenderGraph;

impl RenderGraph {
    pub fn add_pass(&mut self, pass: RenderPassHandle) {}
}
