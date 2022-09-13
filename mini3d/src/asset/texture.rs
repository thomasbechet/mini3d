use super::Asset;

pub struct Texture {
    pub data: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

impl Asset for Texture {
    fn typename() -> &'static str {
        "texture"
    }

    fn default() -> Self {
        Self {
            data: Default::default(),
            width: 0,
            height: 0,
        }
    }
}