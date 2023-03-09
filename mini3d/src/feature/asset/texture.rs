use serde::{Serialize, Deserialize};

use crate::{registry::asset::Asset, uid::UID};

#[derive(Clone, Serialize, Deserialize)]
pub enum TextureFormat {
    R,
    RG,
    RGB,
    RGBA,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Texture {
    pub data: Vec<u8>,
    pub format: TextureFormat,
    pub width: u32,
    pub height: u32,
}

impl Asset for Texture {}

impl Texture {
    pub const NAME: &'static str = "texture";
    pub const UID: UID = UID::new(Texture::NAME);
}