use serde::{Serialize, Deserialize};
use super::Asset;

#[derive(Default, Serialize, Deserialize)]
pub struct Texture {
    pub data: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

impl Asset for Texture {
    fn typename() -> &'static str { "texture" }
}