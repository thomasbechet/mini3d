use glam::IVec2;
use mini3d_derive::Serialize;

use crate::{
    asset::handle::StaticAsset,
    feature::{renderer::texture::Texture, ui::ui_stylesheet::UIStyleSheet},
    math::rect::IRect,
    renderer::{
        color::Color,
        graphics::{Graphics, TextureWrapMode},
    },
    ui::event::{Event, EventContext},
};

use super::Widget;

#[derive(Serialize)]
pub struct UISprite {
    texture: StaticAsset<Texture>,
    color: Color,
    position: IVec2,
    texture_extent: IRect,
}

impl UISprite {
    pub fn new(texture: StaticAsset<Texture>, position: IVec2, texture_extent: IRect) -> Self {
        Self {
            texture,
            color: Color::WHITE,
            position,
            texture_extent,
        }
    }

    pub fn set_position(&mut self, position: IVec2) -> &mut Self {
        self.position = position;
        self
    }

    pub fn set_texture_extent(&mut self, texture_extent: IRect) -> &mut Self {
        self.texture_extent = texture_extent;
        self
    }

    pub fn set_color(&mut self, color: Color) -> &mut Self {
        self.color = color;
        self
    }
}

impl Widget for UISprite {
    fn handle_event(&mut self, _ctx: &mut EventContext, _event: &Event) -> bool {
        false
    }

    fn render(&self, gfx: &mut Graphics, styles: &UIStyleSheet, offset: IVec2, _time: f64) {
        let extent = IRect::new(
            self.position.x + offset.x,
            self.position.y + offset.y,
            self.texture_extent.width(),
            self.texture_extent.height(),
        );
        gfx.blit_texture(
            self.texture,
            extent,
            self.texture_extent,
            self.color,
            TextureWrapMode::Clamp,
            5,
        );
    }

    fn extent(&self) -> IRect {
        self.texture_extent.translate(self.position)
    }

    fn is_focusable(&self) -> bool {
        false
    }

    fn is_selectable(&self) -> bool {
        false
    }
}
