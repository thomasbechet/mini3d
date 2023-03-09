use glam::IVec2;
use serde::{Serialize, Deserialize};

use crate::{renderer::graphics::Graphics, uid::UID, ecs::entity::Entity};

#[derive(Serialize, Deserialize)]
pub struct Viewport {
    pub position: IVec2,
    pub world: UID,
    pub viewport: Entity,
}

impl Viewport {

    pub fn new(position: IVec2, world: UID, viewport: Entity) -> Self {
        Self { position, world, viewport }
    }

    pub fn draw(&self, gfx: &mut Graphics) {
        gfx.blit_viewport(self.world, self.viewport, self.position);   
    }
}