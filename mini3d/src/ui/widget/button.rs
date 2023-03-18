use serde::{Serialize, Deserialize};

use crate::{ui::event::{EventContext, Event}, renderer::{graphics::Graphics, color::Color}, math::rect::IRect, uid::UID};

use super::Widget;

#[derive(Default, Serialize, Deserialize)]
pub struct Button {
    pressed: bool,
    hovered: bool,
    extent: IRect,
    pressed_action: UID,
}

impl Button {
    
    pub fn new(extent: IRect) -> Self {
        Self { extent, ..Default::default() }
    }

    pub(crate) fn set_pressed_action(&mut self, uid: UID) {
        self.pressed_action = uid;
    }
}

impl Widget for Button {

    fn handle_event(&mut self, ctx: &mut EventContext, event: &Event) -> bool {
        match event {
            Event::PrimaryJustPressed => {
                if !self.pressed {
                    self.pressed = true;
                    println!("pressed {:?}", self.pressed_action);
                }
            },
            Event::PrimaryJustReleased => {
                if self.pressed {
                    self.pressed = false;
                    println!("released {:?}", self.pressed_action);
                }
            },
            Event::Enter => {
                self.hovered = true;
            },
            Event::Leave => {
                self.hovered = false;
                if self.pressed {
                    println!("released {:?}", self.pressed_action);
                    self.pressed = false;
                }
            },
            _ => {},
        }
        true
    }

    fn render(&self, gfx: &mut Graphics, time: f64) {
        if self.pressed {
            gfx.draw_rect(self.extent, Color::RED);
        } else if self.hovered {
            gfx.draw_rect(self.extent, Color::GREEN);
        } else {
            gfx.draw_rect(self.extent, Color::WHITE);
        }
    }

    fn extent(&self) -> IRect {
        self.extent
    }

    fn is_focusable(&self) -> bool {
        false
    }
}