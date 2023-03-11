use std::collections::HashMap;

use glam::Vec2;
use serde::{Serialize, Deserialize};

use crate::{feature::component::viewport::Viewport, uid::UID, renderer::graphics::Graphics, math::rect::IRect};

use self::{button::Button, checkbox::Checkbox, sprite::Sprite, label::Label, layout::Layout};

use super::WidgetEntry;

pub mod button;
pub mod checkbox;
pub mod graphics;
pub mod label;
pub mod layout;
pub mod slider;
pub mod sprite;
pub mod viewport;

#[derive(Serialize, Deserialize)]
enum Widget {
    Button(Button),
    Checkbox(Checkbox),
    Label(Label),
    Sprite(Sprite),
    Viewport(Viewport),
    Layout(Layout),
}

impl Widget {

    pub(crate) fn intersect(pos: Vec2) -> Option<UID> {
        None
    }

    pub(crate) fn handle_event(&mut self) {

    }

    pub(crate) fn draw(&self, gfx: &mut Graphics, widgets: &HashMap<UID, WidgetEntry>) {
        match self {
            Widget::Button(_) => todo!(),
            Widget::Checkbox(_) => todo!(),
            Widget::Label(_) => todo!(),
            Widget::Sprite(_) => todo!(),
            Widget::Viewport(_) => todo!(),
            Widget::Layout(_) => todo!(),
        }
    }
}