use crate::{define_resource_handle, renderer::color::Color, utils::string::AsciiArray};

use super::{buffer::BufferHandle, texture::TextureHandle};

define_resource_handle!(GraphicsPassHandle);
define_resource_handle!(CanvasPassHandle);
define_resource_handle!(ComputePassHandle);
define_resource_handle!(CopyPassHandle);

pub(crate) struct ColorAttachment {
    pub(crate) name: AsciiArray<32>,
    pub(crate) texture: TextureHandle,
    pub(crate) clear: Option<Color>,
}

pub(crate) struct DepthStencilAttachment {
    pub(crate) clear_depth: Option<f32>,
    pub(crate) clear_stencil: Option<u32>,
}

pub(crate) enum ShaderResourceKind {
    Texture(TextureHandle),
    Buffer(BufferHandle),
}

pub(crate) struct ShaderResource {
    pub(crate) name: AsciiArray<32>,
    pub(crate) binding: u32,
    pub(crate) kind: ShaderResourceKind,
}

pub struct GraphicsPass {
    shader_resources: Vec<ShaderResource>,
    color_attachments: Vec<ColorAttachment>,
    depth_stencil: Option<DepthStencilAttachment>,
}

impl GraphicsPass {
    pub fn new() -> Self {
        Self {
            shader_resources: Vec::new(),
            color_attachments: Vec::new(),
            depth_stencil: None,
        }
    }

    pub fn with_shader_resource(mut self, binding: ShaderResource) -> Self {
        self.shader_resources.push(resource);
        self
    }

    pub fn with_color_attachment(mut self, attachment: ColorAttachment) -> Self {
        self.color_attachments.push(attachment);
        self
    }
}

pub struct CanvasPass {
    target: ColorAttachment,
}

impl CanvasPass {}

pub struct ComputePass {}

impl ComputePass {}

pub struct CopyPass {}

impl CopyPass {}
