use mini3d_derive::{Error, Reflect, Serialize};

use crate::{define_resource_handle, feature::core::resource::Resource};

#[derive(Error)]
pub enum RenderGraphError {
    #[error("Failed to compile graph")]
    CompilationError,
}

#[derive(Default, Serialize, Reflect)]
pub(crate) enum RenderPass {
    #[default]
    Graphics,
    Compute,
    Copy,
}

#[derive(Default, Serialize, Reflect)]
pub(crate) struct RenderGraph {
    passes: Vec<RenderPass>,
}

impl RenderGraph {
    pub const NAME: &'static str = "RTY_RenderGraph";

    pub(crate) fn build(&mut self) {}
}

impl Resource for RenderGraph {}

define_resource_handle!(RenderGraphHandle);
