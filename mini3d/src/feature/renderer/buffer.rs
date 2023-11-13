use mini3d_derive::{Reflect, Serialize};

use crate::{feature::core::resource::Resource, renderer::provider::RendererProviderHandle};

#[derive(Clone, Serialize, Reflect)]
pub enum AttributeFormat {
    I8x2,
    I8x4,
    I16x2,
    I16x4,
    I32x2,
    I32x4,
    F32x2,
    F32x4,
    M4x4,
}

#[derive(Default, Serialize, Reflect)]
pub enum RenderBufferType {
    #[default]
    Uniform,
    Vertex,
    Instance,
}

#[derive(Default, Serialize, Reflect)]
pub enum RenderBufferUsage {
    #[default]
    Static,
    Dynamic,
}

pub struct RenderBufferDesc {
    ty: RenderBufferType,
    usage: RenderBufferUsage,
    attributes: [AttributeFormat; RenderBuffer::MAX_ATTRIBUTE],
    attribute_count: u8,
    size: u16,
}

#[derive(Default, Serialize, Reflect)]
pub struct RenderBuffer {
    ty: RenderBufferType,
    usage: RenderBufferUsage,
    attributes: [AttributeFormat; RenderBuffer::MAX_ATTRIBUTE],
    attribute_count: u8,
    size: u16,
    #[serialize(skip)]
    handle: RendererProviderHandle,
}

impl RenderBuffer {
    pub const MAX_ATTRIBUTE: usize = 8;
    pub const MAX_SIZE: usize = 65535;
}

pub struct RenderBufferHandle(u16);
