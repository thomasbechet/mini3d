use crate::resource::handle::ResourceHandle;

use super::render_pass::RenderPassHandle;

pub enum ResourceAccess {
    ReadOnly,
    WriteOnly,
    ReadWrite,
}

pub struct RenderGraphHandle(pub(crate) ResourceHandle);

pub struct RenderGraph;

impl RenderGraph {
    pub fn add_render_pass(&mut self, pass: RenderPassHandle) {}
    pub fn add_compute_pass(&mut self, pass: ComputePas) {}
    pub fn build();
}
