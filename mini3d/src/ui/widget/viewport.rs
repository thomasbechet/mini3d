use glam::IVec2;
use serde::{Serialize, Deserialize};

use crate::{renderer::graphics::Graphics, uid::UID, ecs::entity::Entity, math::rect::IRect, ui::event::{EventContext, Event}};

use super::Widget;

#[derive(Serialize, Deserialize)]
pub struct UIViewport {
    pub position: IVec2,
    pub world: UID,
    pub viewport: Entity,
}

impl UIViewport {

    pub fn new(position: IVec2, world: UID, viewport: Entity) -> Self {
        Self { position, world, viewport }
    }
}

impl Widget for UIViewport {

    fn handle_event(&mut self, _ctx: &mut EventContext, _event: &Event) -> bool {
        true
    }

    fn render(&self, gfx: &mut Graphics, offset: IVec2, _time: f64) {
        gfx.blit_viewport(self.world, self.viewport, self.position + offset);   

    }

    fn extent(&self) -> IRect {
        IRect::new(0, 0, 10, 10)
    }

    fn is_focusable(&self) -> bool {
        false
    }

    fn is_selectable(&self) -> bool {
        false
    }
}