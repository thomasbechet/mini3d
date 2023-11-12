use crate::{
    feature::renderer::graph::{RenderGraphFeaturesHandle, RenderGraphHandle},
    resource::ResourceManager,
};

use super::queue::GraphicsQueue;

struct GraphicsPassEntry {
    queue: GraphicsQueue,
}

pub(crate) struct FrameGraphInstance {
    pub(crate) graphics_queues: Vec<GraphicsQueue>,
}

impl FrameGraphInstance {
    pub(crate) fn compile(
        &mut self,
        resources: &mut ResourceManager,
        graph: RenderGraphHandle,
        features: RenderGraphFeaturesHandle,
    ) {
    }
}
