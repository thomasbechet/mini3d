use glam::UVec2;
use mini3d_derive::{Error, Reflect, Serialize};

use crate::{
    define_resource_handle, feature::core::resource::Resource, renderer::SCREEN_RESOLUTION,
    utils::string::AsciiArray,
};

use super::{
    buffer::{BufferUsage, RenderBufferHandle},
    pass::RenderTarget,
    texture::{TextureFormat, TextureHandle},
};

#[derive(Error)]
pub enum RenderGraphError {
    #[error("Failed to compile graph")]
    CompilationError,
}

#[derive(Default, Serialize, Reflect)]
pub(crate) enum RenderPassKind {
    #[default]
    Graphics,
    Compute,
    Copy,
}

struct RenderPassEntry {
    kind: RenderPassKind,
    first_resource: usize,
    resource_count: usize,
}

pub(crate) enum ResourceKind {
    Buffer {
        size: usize,
        usage: BufferUsage,
    },
    Texture {
        resolution: UVec2,
        format: TextureFormat,
    },
}

enum ResourceUsage {
    Read,
    Write,
    Create,
}

struct ResourceReferenceEntry {
    usage: ResourceUsage,
    index: usize,
}

struct ResourceEntry {
    name: AsciiArray<32>,
    ref_count: usize,
    kind: ResourceKind,
}

pub(crate) struct GraphicsPassBuilder<'a> {}

impl<'a> GraphicsPassBuilder<'a> {
    pub fn read(mut self, name: &str, resource: ResourceKind) -> Self {
        self
    }
}

pub(crate) struct ComputePassBuilder<'a> {}

impl<'a> ComputePassBuilder<'a> {
    pub fn read(mut self, name: &str, resource: ResourceKind) -> Self {
        self
    }

    pub fn write(mut self, name: &str, resource: ResourceKind) -> Self {
        self
    }

    pub fn create(mut self, name: &str, resource: ResourceKind) -> Self {
        self
    }
}

#[derive(Default, Serialize, Reflect)]
pub(crate) struct RenderGraph {
    passes: Vec<RenderPassEntry>,
    resource_references: Vec<ResourceReferenceEntry>,
    resources: Vec<ResourceEntry>,
    target: usize,
}

impl RenderGraph {
    pub const NAME: &'static str = "RTY_RenderGraph";

    pub(crate) fn new() -> Self {
        let mut graph = Self::default();
        graph.resources.push(ResourceEntry {
            name: "target".into(),
            ref_count: 1,
            kind: ResourceKind::Texture {
                resolution: SCREEN_RESOLUTION,
                format: TextureFormat::RGBA,
            },
        });
        graph
    }

    pub(crate) fn set_target(&mut self, target: usize) {}

    pub(crate) fn add_graphics_pass(&'_ mut self, name: &str) -> RenderGraphBuilder<'_> {}

    pub(crate) fn compile(&mut self) {
        // Reset reference count
        for resource in self.resources.iter_mut() {
            resource.ref_count = 0;
        }
    }
}

impl Resource for RenderGraph {}

define_resource_handle!(RenderGraphHandle);

#[derive(Default, Serialize, Reflect)]
pub(crate) struct RenderGraphFeatures {}

impl RenderGraphFeatures {
    pub const NAME: &'static str = "RTY_RenderGraphFeatures";
}

impl Resource for RenderGraphFeatures {}

define_resource_handle!(RenderGraphFeaturesHandle);
