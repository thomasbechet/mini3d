use anyhow::Result;
use glam::IVec2;
use serde::{Serialize, Deserialize};

use crate::{uid::UID, renderer::graphics::Graphics, ui::event::{EventContext, Event}, math::rect::IRect, feature::asset::ui_stylesheet::UIStyleSheet};

use super::Widget;

#[derive(Serialize, Deserialize)]
pub struct UILabel {
    position: IVec2,
    text: String,
    font: UID,
}

impl UILabel {

    pub fn new(position: IVec2, text: &str, font: UID) -> Self {
        Self { position, text: text.to_owned(), font }
    }
}

impl Widget for UILabel {

    fn handle_event(&mut self, ctx: &mut EventContext, _event: &Event) -> bool {
        true
    }

    fn render(&self, gfx: &mut Graphics, styles: &UIStyleSheet, offset: IVec2, _time: f64) -> Result<()> {
        gfx.print(self.position + offset, &self.text, self.font);
        Ok(())
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