use crate::{math::rect::IRect, uid::UID};
use glam::IVec2;
use mini3d_derive::{Asset, Error};

#[derive(Debug, Error)]
pub enum TilesetError {
    #[error("Invalid tile index")]
    InvalidTileIndex,
}

#[derive(Clone, Asset)]
pub struct Tileset {
    pub texture: UID,
    pub offset: IVec2,
    pub tile_width: u32,
    pub tile_height: u32,
    pub grid_width: u32,
    pub grid_height: u32,
}

impl Tileset {
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
            self.tile_height,
        ))
    }
}
