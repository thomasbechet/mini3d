use serde::{Serialize, Deserialize};

use crate::uid::UID;

#[derive(Clone, Serialize, Deserialize)]
pub struct TilemapAsset {
    pub tileset: UID,
    pub tiles: Vec<u32>,
    pub width: u32,
    pub height: u32,
}