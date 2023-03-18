use glam::IVec2;
use serde::{Serialize, Deserialize};

use crate::{uid::UID, renderer::{color::Color, graphics::Graphics}, math::rect::IRect, ui::event::{EventContext, Event}};

use super::Widget;

#[derive(Serialize, Deserialize)]
pub struct TextBox {
    extent: IRect,
    focused: bool,
}

impl TextBox {

    pub fn new(extent: IRect) -> Self {
        Self {
            extent,
            focused: false,
        }
    }
}

impl Widget for TextBox {

    fn handle_event(&mut self, ctx: &mut EventContext, event: &Event) -> bool {
        match event {
            Event::PrimaryJustPressed => {},
            Event::PrimaryJustReleased => {},
            Event::SecondaryJustPressed => {},
            Event::SecondaryJustReleased => {},
            Event::Enter => {},
            Event::Leave => {},
            Event::GainFocus => {
                println!("focus");
                self.focused = true
            },
            Event::LooseFocus => {
                println!("unfocus");
                self.focused = false
            },
            Event::Text { value } => todo!(),
            Event::Scroll { value } => todo!(),
            Event::SelectionMove { direction } => todo!(),
            Event::CursorMove { position } => {},
            Event::ModeChange => todo!(),
        }
        true
    }

    fn render(&self, gfx: &mut Graphics, time: f64) {
        gfx.draw_rect(self.extent, Color::WHITE);
        if self.focused {
            let extent = IRect::new(
                self.extent.left() + 2, 
                self.extent.top() + 2, 
                self.extent.width() - 4, 
                self.extent.height() - 4
            );
            gfx.fill_rect(extent, Color::RED);
        }
    }

    fn extent(&self) -> IRect {
        self.extent
    }

    fn is_focusable(&self) -> bool { true }
}