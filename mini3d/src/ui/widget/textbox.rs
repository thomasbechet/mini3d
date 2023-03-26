use anyhow::Result;
use glam::IVec2;
use serde::{Serialize, Deserialize};

use crate::{renderer::{color::Color, graphics::Graphics}, math::rect::IRect, ui::event::{EventContext, Event}, feature::asset::ui_stylesheet::UIStyleSheet};

use super::Widget;

#[derive(Serialize, Deserialize)]
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

    fn render(&self, gfx: &mut Graphics, styles: &UIStyleSheet, offset: IVec2, time: f64) -> Result<()> {
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
        Ok(())
    }

    fn extent(&self) -> IRect {
        self.extent
    }

    fn is_focusable(&self) -> bool { true }

    fn is_selectable(&self) -> bool { true }
}