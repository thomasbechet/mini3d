use glam::IVec2;
use hecs::Entity;
use serde::{Serialize, Deserialize};

use crate::{renderer::graphics::Graphics, uid::UID};

#[derive(Serialize, Deserialize)]
pub struct Viewport {
    pub position: IVec2,
    pub scene: UID,
    pub viewport: Entity,
}

impl Viewport {

    pub fn new(position: IVec2, scene: UID, viewport: Entity) -> Self {
        Self { position, scene, viewport }
    }

    pub fn draw(&self, gfx: &mut Graphics) {
        gfx.blit_viewport(self.scene, self.viewport, self.position);   
    }
}