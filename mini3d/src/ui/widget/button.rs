use glam::IVec2;
use serde::{Serialize, Deserialize};

use crate::{ui::{event::{EventContext, Event, UIEvent}, style::UIBoxStyle}, renderer::{graphics::Graphics, color::Color}, math::rect::IRect, uid::UID};

use super::Widget;

#[derive(Serialize, Deserialize)]
pub struct UIButtonStyle {
    normal: UIBoxStyle,
    pressed: UIBoxStyle,
    hovered: UIBoxStyle,
}

impl UIButtonStyle {
    pub fn new(normal: UIBoxStyle, pressed: UIBoxStyle, hovered: UIBoxStyle) -> Self {
        Self { normal, pressed, hovered }
    }
}

#[derive(Serialize, Deserialize)]
pub struct UIButton {
    pressed: bool,
    hovered: bool,
    extent: IRect,
    style: UIButtonStyle,
    on_pressed: Option<UID>,
    on_released: Option<UID>,
}

impl UIButton {

    pub fn new(extent: IRect, style: UIButtonStyle) -> Self {
        Self { extent, style, pressed: false, hovered: false, on_pressed: None, on_released: None }
    }

    pub fn on_pressed(&mut self, action: Option<UID>) {
        self.on_pressed = action;
    }

    pub fn on_released(&mut self, action: Option<UID>) {
        self.on_released = action;
    }
}

impl Widget for UIButton {

    fn handle_event(&mut self, ctx: &mut EventContext, event: &Event) -> bool {
        match event {
            Event::PrimaryJustPressed => {
                if !self.pressed {
                    self.pressed = true;
                    if let Some(action) = self.on_pressed {
                        ctx.events.push(UIEvent::Action { user: ctx.user.uid(), id: action });
                    }
                }
            },
            Event::PrimaryJustReleased => {
                if self.pressed {
                    self.pressed = false;
                    if let Some(action) = self.on_released {
                        ctx.events.push(UIEvent::Action { user: ctx.user.uid(), id: action });
                    }
                }
            },
            Event::Enter => {
                self.hovered = true;
            },
            Event::Leave => {
                self.hovered = false;
                if self.pressed {
                    if let Some(action) = self.on_released {
                        ctx.events.push(UIEvent::Action { user: ctx.user.uid(), id: action });
                    }
                    self.pressed = false;
                }
            },
            _ => {},
        }
        true
    }

    fn render(&self, gfx: &mut Graphics, offset: IVec2, _time: f64) {
        let extent = self.extent.translate(offset);
        if self.pressed {
            self.style.pressed.render(gfx, extent, Color::WHITE, 0);
        } else if self.hovered {
            self.style.hovered.render(gfx, extent, Color::WHITE, 0);
        } else {
            self.style.normal.render(gfx, extent, Color::WHITE, 0);
        }
    }

    fn extent(&self) -> IRect {
        self.extent
    }

    fn is_focusable(&self) -> bool {
        false
    }

    fn is_selectable(&self) -> bool { true }
}