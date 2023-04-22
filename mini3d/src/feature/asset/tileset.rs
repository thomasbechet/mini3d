use std::{error::Error, fmt::Display};

use glam::IVec2;
use serde::{Serialize, Deserialize};

use crate::{uid::UID, math::rect::IRect, registry::asset::Asset};

#[derive(Debug)]
pub enum TilesetError {
    InvalidTileIndex,
}

impl Error for TilesetError {}

impl Display for TilesetError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TilesetError::InvalidTileIndex => write!(f, "Invalid tile index"),   
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Tileset {
    pub texture: UID,
    pub offset: IVec2,
    pub tile_width: u32,
    pub tile_height: u32,
    pub grid_width: u32,
    pub grid_height: u32,
}

impl Asset for Tileset {}

impl Tileset {

    pub const NAME: &'static str = "tileset";
    pub const UID: UID = UID::new(Tileset::NAME);
    
    pub fn extent(&self, tile: u32) -> Result<IRect, TilesetError> {
        if tile >= self.grid_width * self.grid_height {
            return Err(TilesetError::InvalidTileIndex);
        }
        let x = tile % self.grid_width;
        let y = tile / self.grid_height;
        Ok(IRect::new(
            self.offset.x + (x * self.tile_width) as i32, 
            self.offset.y + (y * self.tile_height) as i32, 
            self.tile_width,
            self.tile_height
        ))
    }
}