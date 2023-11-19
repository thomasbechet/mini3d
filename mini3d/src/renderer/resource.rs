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

pub enum GPUTextureUsage {
    RenderTarget,
}

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
