use serde::{Serialize, Deserialize};

#[derive(Clone, Serialize, Deserialize)]
pub enum TextureFormat {
    R,
    RG,
    RGB,
    RGBA,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct TextureAsset {
    pub data: Vec<u8>,
    pub format: TextureFormat,
    pub width: u32,
    pub height: u32,
}