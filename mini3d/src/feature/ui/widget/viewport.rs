use glam::IVec2;
use mini3d_derive::Serialize;

use crate::{
    ecs::entity::Entity,
    feature::ui::ui_stylesheet::UIStyleSheet,
    math::rect::IRect,
    renderer::graphics::Graphics,
    ui::event::{Event, EventContext},
};

use super::Widget;

#[derive(Serialize)]
pub struct UIViewport {
    pub position: IVec2,
    pub viewport: Entity,
}

impl UIViewport {
    pub fn new(position: IVec2, viewport: Entity) -> Self {
        Self { position, viewport }
    }
}

impl Widget for UIViewport {
    fn handle_event(&mut self, _ctx: &mut EventContext, _event: &Event) -> bool {
        true
    }

    fn render(&self, gfx: &mut Graphics, styles: &UIStyleSheet, offset: IVec2, _time: f64) {
        gfx.blit_viewport(self.viewport, self.position + offset);
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