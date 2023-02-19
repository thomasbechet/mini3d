use serde::{Serialize, Deserialize};

use crate::{uid::UID, registry::asset::Asset};

#[derive(Clone, Serialize, Deserialize)]
pub struct Tilemap {
    pub tileset: UID,
    pub tiles: Vec<u32>,
    pub width: u32,
    pub height: u32,
}

impl Asset for Tilemap {}

impl Tilemap {
    pub const NAME: &'static str = "tilemap";
    pub const UID: UID = UID::new(Tilemap::NAME);
}