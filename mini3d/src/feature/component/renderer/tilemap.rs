use mini3d_derive::{Component, Reflect, Serialize};

use crate::uid::UID;

#[derive(Clone, Component, Serialize, Default, Reflect)]
pub struct Tilemap {
    pub tileset: UID,
    pub tiles: Vec<u32>,
    pub width: u32,
    pub height: u32,
}
