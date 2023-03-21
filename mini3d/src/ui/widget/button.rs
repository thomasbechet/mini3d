use glam::IVec2;
use serde::{Serialize, Deserialize};

use crate::{ui::event::{EventContext, Event, UIEvent}, renderer::{graphics::Graphics, color::Color}, math::rect::IRect, uid::UID};

use super::Widget;

pub enum ButtonEvent {
    Pressed,
    Released,
}

#[derive(Default, Serialize, Deserialize)]
pub struct Button {
    pressed: bool,
    hovered: bool,
    extent: IRect,
    on_pressed: Option<UID>,
    on_released: Option<UID>,
}

impl Button {

    pub fn new(extent: IRect) -> Self {
        Self { extent, ..Default::default() }
    }

    pub fn on_pressed(&mut self, action: Option<UID>) {
        self.on_pressed = action;
    }

    pub fn on_released(&mut self, action: Option<UID>) {
        self.on_released = action;
    }
}

impl Widget for Button {

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
            gfx.draw_rect(extent, Color::RED);
        } else if self.hovered {
            gfx.draw_rect(extent, Color::GREEN);
        } else {
            gfx.draw_rect(extent, Color::WHITE);
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