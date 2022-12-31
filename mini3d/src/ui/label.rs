use glam::IVec2;
use serde::{Serialize, Deserialize};

use crate::{uid::UID, renderer::graphics::Graphics};

#[derive(Serialize, Deserialize)]
pub struct Label {
    position: IVec2,
    text: String,
    font: UID,
}

impl Label {

    pub fn new(position: IVec2, text: &str, font: UID) -> Self {
        Self { position, text: text.to_owned(), font }
    }

    pub fn draw(&self, gfx: &mut Graphics) {
        gfx.print(self.position, &self.text, self.font);
    }
}