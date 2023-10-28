use mini3d_derive::Error;

use crate::define_resource_handle;

define_resource_handle!(RenderGraphHandle);

#[derive(Error)]
pub enum RenderGraphError {
    #[error("Failed to compile graph")]
    CompilationError,
}

pub(crate) enum RenderPass {
    Graphics,
    Compute,
    Copy,
}

pub(crate) struct RenderGraph {
    passes: Vec<RenderPass>,
}

impl RenderGraph {
    pub(crate) fn build(&mut self) {}
}
