use anyhow::{Result, anyhow};
use glam::IVec2;
use serde::{Serialize, Deserialize};

use crate::{uid::UID, math::rect::IRect, registry::asset::Asset};

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
    
    pub fn extent(&self, tile: u32) -> Result<IRect> {
        if tile >= self.grid_width * self.grid_height {
            return Err(anyhow!("Invalid tile index"));
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