use mini3d_derive::{Asset, Serialize};

#[derive(Clone, Serialize)]
pub enum TextureFormat {
    R,
    RG,
    RGB,
    RGBA,
}

#[derive(Clone, Asset)]
pub struct Texture {
    pub data: Vec<u8>,
    pub format: TextureFormat,
    pub width: u32,
    pub height: u32,
}
