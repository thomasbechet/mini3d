use crate::{renderer::color::Color, utils::string::AsciiArray};

use super::{buffer::BufferHandle, texture::TextureHandle};

pub(crate) struct RenderTarget {
    pub(crate) name: AsciiArray<32>,
    pub(crate) texture: TextureHandle,
    pub(crate) clear: Option<Color>,
}

pub(crate) struct DepthStencilAttachment {
    pub(crate) clear_depth: Option<f32>,
    pub(crate) clear_stencil: Option<u32>,
}

pub(crate) enum RenderPassResourceKind {
    Texture(TextureHandle),
    UniformBuffer(BufferHandle),
}

pub(crate) struct RenderPassResource {
    pub(crate) name: AsciiArray<32>,
    pub(crate) binding: u32,
    pub(crate) kind: RenderPassResourceKind,
}

pub struct GraphicsPass {
    resources: Vec<RenderPassResource>,
    render_targets: Vec<RenderTarget>,
    depth_stencil: Option<DepthStencilAttachment>,
}

impl GraphicsPass {
    pub fn new() -> Self {
        Self {
            resources: Vec::new(),
            render_targets: Vec::new(),
            depth_stencil: None,
        }
    }

    // pub fn with_shader_resource(mut self, binding: ShaderResource) -> Self {
    //     self.shader_resources.push(resource);
    //     self
    // }

    // pub fn with_color_attachment(mut self, attachment: ColorAttachment) -> Self {
    //     self.color_attachments.push(attachment);
    //     self
    // }
}

pub struct CanvasPass {
    target: RenderTarget,
}

impl CanvasPass {}

pub struct ComputePass {}

impl ComputePass {}

pub struct CopyPass {}

impl CopyPass {}
