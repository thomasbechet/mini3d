use super::{color::Color, resource::GPUTextureFormat};

pub(crate) enum RenderPassResource {}

pub(crate) struct ColorAttachment {
    clear: Option<Color>,
    format: GPUTextureFormat,
}

pub(crate) struct DepthStencilAttachment {
    pub(crate) clear_depth: Option<f32>,
    pub(crate) clear_stencil: Option<u32>,
}

pub(crate) struct GraphicsPassLayout {
    color_attachments: [ColorAttachment; 4],
    depth_stencil: Option<DepthStencilAttachment>,
    resources: [RenderPassResource; 16],
    resource_layouts: [u8; 8],
}

pub(crate) struct GraphicsPass {
    layout: GraphicsPassLayout,
}

pub(crate) struct ComputePassLayout {
    resources: [RenderPassResource; 16],
}

pub(crate) struct ComputePass {
    layout: ComputePassLayout,
}
