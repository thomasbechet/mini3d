use crate::{
    feature::renderer::graph::{RenderGraphFeaturesHandle, RenderGraphHandle},
    resource::ResourceManager,
};

use super::queue::GraphicsQueue;

struct GraphicsPassEntry {
    queue: GraphicsQueue,
}

pub(crate) struct RenderGraphInstance {
    pub(crate) graphics_queues: Vec<GraphicsQueue>,
}

impl RenderGraphInstance {
    pub(crate) fn compile(
        &mut self,
        resources: &mut ResourceManager,
        graph: RenderGraphHandle,
        features: RenderGraphFeaturesHandle,
    ) {
    }
}
