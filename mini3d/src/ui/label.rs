use anyhow::{Result, Context};
use glam::IVec2;
use serde::{Serialize, Deserialize};

use crate::{uid::UID, renderer::{backend::RendererBackend, RendererResourceManager, color::Color}, asset::AssetManager};

#[derive(Serialize, Deserialize)]
pub struct Label {
    position: IVec2,
    text: String,
    font: UID,
}

impl Label {

    pub(crate) fn draw(
        &mut self, 
        resources: &mut RendererResourceManager,
        backend: &mut impl RendererBackend,
        asset: &AssetManager
    ) -> Result<()> {
        let font = resources.request_font(&self.font, backend, asset)?;
        let mut position = self.position;
        for c in self.text.chars() {
            let extent = font.atlas.extents.get(&c).with_context(|| "Character extent not found")?;
            backend.canvas_blit_rect(font.handle, *extent, position, Color::WHITE, 1)?;
            position.x += extent.width() as i32;
        }
        Ok(())
    }

    pub fn new(position: IVec2, text: &str, font: UID) -> Self {
        Self { position, text: text.to_owned(), font }
    }
}