use mini3d_derive::{Component, Reflect, Serialize};

#[derive(Clone, Serialize, Default)]
pub enum TextureFormat {
    R,
    RG,
    RGB,
    #[default]
    RGBA,
}

#[derive(Clone, Component, Serialize, Default, Reflect)]
pub struct Texture {
    pub data: Vec<u8>,
    pub format: TextureFormat,
    pub width: u32,
    pub height: u32,
}