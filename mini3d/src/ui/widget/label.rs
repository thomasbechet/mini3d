use glam::IVec2;
use mini3d_derive::Serialize;

use crate::{
    feature::component::ui::ui_stylesheet::UIStyleSheet,
    math::rect::IRect,
    renderer::graphics::Graphics,
    ui::event::{Event, EventContext},
    uid::UID,
};

use super::Widget;

#[derive(Serialize)]
pub struct UILabel {
    position: IVec2,
    text: String,
    font: UID,
}

impl UILabel {
    pub fn new(position: IVec2, text: &str, font: UID) -> Self {
        Self {
            position,
            text: text.to_owned(),
            font,
        }
    }
}

impl Widget for UILabel {
    fn handle_event(&mut self, ctx: &mut EventContext, _event: &Event) -> bool {
        true
    }

    fn render(&self, gfx: &mut Graphics, styles: &UIStyleSheet, offset: IVec2, _time: f64) {
        gfx.print(self.position + offset, &self.text, self.font);
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
