use serde::{Serialize, Deserialize};

use super::Asset;

#[derive(Default, Clone, Serialize, Deserialize)]
pub struct Texture {
    pub data: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

impl Asset for Texture {}