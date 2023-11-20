use mini3d_derive::{Error, Reflect, Serialize};

use crate::{
    define_resource_handle, feature::core::resource::Resource, renderer::color::Color,
    slot_map_key, utils::string::AsciiArray,
};

use super::{
    array::{RenderArrayHandle, RenderFormat},
    texture::{TextureFormat, TextureHandle},
};

#[derive(Error)]
pub enum RenderGraphError {
    #[error("Failed to compile graph")]
    CompilationError,
}

pub enum RenderTarget {
    Texture(TextureHandle),
    Array(RenderArrayHandle),
}

slot_map_key!(RenderGraphSlot);

pub enum RenderPassResource {
    Texture { format: TextureFormat },
    Array { format: RenderFormat, size: u32 },
    Constant { format: RenderFormat },
}

pub(crate) struct ColorAttachment {
    clear: Option<Color>,
    format: TextureFormat,
}

pub(crate) struct DepthStencilAttachment {
    pub(crate) clear_depth: Option<f32>,
    pub(crate) clear_stencil: Option<u32>,
}

pub(crate) struct GraphicsPassLayout {
    color_attachments: [ColorAttachment; 4],
    depth_stencil: Option<DepthStencilAttachment>,
    resources: [RenderPassResource; 16],
}

#[derive(Default, Serialize, Reflect)]
pub(crate) enum RenderPassKind {
    #[default]
    Graphics,
    Compute,
    Canvas,
}

#[derive(Default, Serialize, Reflect)]
struct RenderPassEntry {
    name: AsciiArray<32>,
    kind: RenderPassKind,
}

#[derive(Default, Serialize, Reflect)]
pub(crate) struct RenderGraph {
    passes: Vec<RenderPassEntry>,
}

impl RenderGraph {
    pub const NAME: &'static str = "RTY_RenderGraph";
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
