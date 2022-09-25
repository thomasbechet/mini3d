use slotmap::new_key_type;

use super::Asset;

#[derive(Default)]
pub struct Texture {
    pub data: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

new_key_type! { pub struct TextureId; }

impl Asset for Texture {

    type Id = TextureId;

    fn typename() -> &'static str {
        "texture"
    }
}