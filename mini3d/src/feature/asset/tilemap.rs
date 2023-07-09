use mini3d_derive::Asset;

use crate::uid::UID;

#[derive(Clone, Asset)]
pub struct Tilemap {
    pub tileset: UID,
    pub tiles: Vec<u32>,
    pub width: u32,
    pub height: u32,
}
