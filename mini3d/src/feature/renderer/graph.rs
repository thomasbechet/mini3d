use crate::define_resource_handle;

pub enum ResourceAccess {
    ReadOnly,
    WriteOnly,
    ReadWrite,
}

define_resource_handle!(RenderGraphHandle);

pub struct RenderGraph;

impl RenderGraph {
    pub fn add_render_pass(&mut self, pass: RenderPassHandle) {}
    pub fn add_compute_pass(&mut self, pass: ComputePas) {}
    pub fn build();
}
