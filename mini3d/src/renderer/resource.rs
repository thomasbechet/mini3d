use mini3d_derive::{Reflect, Serialize};

pub(crate) enum ShaderResourceType {
    PushConstant,
    Constant,
    ConstantArray,
    Texture,
    TextureCube,
}

pub(crate) enum GPUResourceType {
    Constant,
    Array,
    Texture,
}

pub(crate) struct ShaderLayout {}

pub enum GPUFormat {
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

pub enum GPUTextureFormat {
    R8,
    R8G8,
    R8G8B8,
    R8G8B8A8,
    R16,
    R16G16,
    R16G16B16,
    R16G16B16A16,
    R32,
    R32G32,
    R32G32B32,
    R32G32B32A32,
}

#[derive(Default, Serialize, Reflect)]
pub enum GPUArrayUsage {
    #[default]
    Static,
    Dynamic,
}

pub struct GPUArray {
    format: GPUFormat,
    usage: GPUArrayUsage,
    size: u32,
}

pub struct GPUConstant {
    format: GPUFormat,
}

pub struct GPUTexture {
    format: GPUTextureFormat,
}

pub struct GPUArrayHandle(u16);
pub struct GPUConstantHandle(u16);
pub struct GPUTextureHandle(u16);

pub(crate) struct GPUResourceHandle(u16);

struct ResourceEntry {
    ty: GPUResourceType,
}

pub(crate) struct GPUResourceTable {
    resources: Vec<ResourceEntry>,
    textures: Vec<()>,
    arrays: Vec<()>,
    constants: Vec<()>,
}
