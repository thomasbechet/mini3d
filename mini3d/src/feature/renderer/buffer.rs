use mini3d_derive::{Reflect, Serialize};

use crate::{
    define_resource_handle, feature::core::resource::Resource,
    renderer::provider::RendererProviderHandle,
};

#[derive(Clone, Serialize, Reflect)]
pub enum AttributeFormat {
    Integer,
    Float,
    Vec2,
    Vec3,
    Vec4,
    Mat4,
}

#[derive(Default, Serialize, Reflect)]
pub enum BufferUsage {
    #[default]
    Uniform,
    Vertex,
    Instance,
}

#[derive(Default, Serialize, Reflect)]
pub struct Buffer {
    attributes: Vec<AttributeFormat>,
    size: usize,
    usage: BufferUsage,
    #[serialize(skip)]
    handle: RendererProviderHandle,
}

impl Buffer {
    pub const NAME: &'static str = "RTY_Buffer";
}

impl Resource for Buffer {}

define_resource_handle!(BufferHandle);
