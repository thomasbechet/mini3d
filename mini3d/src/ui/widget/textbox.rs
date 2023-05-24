use glam::IVec2;
use mini3d_derive::Serialize;

use crate::{renderer::{color::Color, graphics::Graphics}, math::rect::IRect, ui::event::{EventContext, Event}, feature::asset::ui_stylesheet::UIStyleSheet};

use super::Widget;

#[derive(Serialize)]
pub struct UITextBox {
    extent: IRect,
    focused: bool,
    text: String,
}

impl UITextBox {

    pub fn new(extent: IRect) -> Self {
        Self {
            extent,
            focused: false,
            text: String::new(),
        }
    }
}

impl Widget for UITextBox {

    fn handle_event(&mut self, ctx: &mut EventContext, event: &Event) -> bool {
        match event {
            Event::PrimaryJustPressed => {},
            Event::PrimaryJustReleased => {},
            Event::Cancel => {},
            Event::Enter => {},
            Event::Leave => {},
            Event::GainFocus => {
                println!("focus");
                self.focused = true;
                ctx.user.locked = true;
            },
            Event::LooseFocus => {
                println!("unfocus");
                self.focused = false;
                ctx.user.locked = false;
            },
            Event::Text { value } => todo!(),
            Event::Scroll { value } => todo!(),
            Event::SelectionMoved { direction } => todo!(),
            Event::CursorMoved { position } => {},
            Event::ModeChanged => todo!(),
        }
        true
    }

    fn render(&self, gfx: &mut Graphics, styles: &UIStyleSheet, offset: IVec2, time: f64) {
        gfx.draw_rect(self.extent.translate(offset), Color::WHITE);
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

    fn is_selectable(&self) -> bool { true }
}