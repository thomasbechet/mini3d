use mini3d_derive::{Asset, Reflect, Serialize};

#[derive(Clone, Serialize, Default)]
pub enum TextureFormat {
    R,
    RG,
    RGB,
    #[default]
    RGBA,
}

#[derive(Clone, Asset, Serialize, Default, Reflect)]
pub struct Texture {
    pub data: Vec<u8>,
    pub format: TextureFormat,
    pub width: u32,
    pub height: u32,
}
