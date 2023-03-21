use glam::IVec2;
use serde::{Serialize, Deserialize};

use crate::{renderer::{color::Color, graphics::Graphics}, math::rect::IRect, uid::UID, ui::event::{EventContext, Event, UIEvent}};

use super::Widget;

#[derive(Serialize, Deserialize)]
pub struct CheckBox {
    position: IVec2,
    checked: bool,
    hovered: bool,
    on_checked: Option<UID>,
    on_unchecked: Option<UID>,
}

impl CheckBox {
    
    pub fn new(position: IVec2, checked: bool) -> Self {
        Self { position, checked, hovered: false, on_checked: None, on_unchecked: None }
    }

    pub fn on_checked(&mut self, action: Option<UID>) {
        self.on_checked = action;
    }

    pub fn on_unchecked(&mut self, action: Option<UID>) {
        self.on_unchecked = action;
    }
}

impl Widget for CheckBox {

    fn handle_event(&mut self, ctx: &mut EventContext, event: &Event) -> bool {
        match event {
            Event::PrimaryJustPressed => {
                self.checked = !self.checked;
                if self.checked {
                    if let Some(action) = self.on_checked {
                        ctx.events.push(UIEvent::Action { user: ctx.user.uid(), id: action });
                    }
                } else if let Some(action) = self.on_unchecked {
                    ctx.events.push(UIEvent::Action { user: ctx.user.uid(), id: action });
                }
            },
            Event::Enter => {
                self.hovered = true;
            },
            Event::Leave => {
                self.hovered = false;
            },
            _ => {},
        }
        true
    }

    fn render(&self, gfx: &mut Graphics, offset: IVec2, _time: f64) {
        let position = self.position + offset;
        gfx.draw_rect(IRect::new(position.x, position.y, 10, 10), Color::WHITE);
        if self.checked {
            gfx.draw_line(position, position + IVec2::new(9, 9), Color::WHITE);
            gfx.draw_line(position + IVec2::new(9, 0), position + IVec2::new(0, 9), Color::WHITE);
        }
    }

    fn extent(&self) -> IRect {
        IRect::new(self.position.x, self.position.y, 10, 10)
    }

    fn is_focusable(&self) -> bool {
        false
    }

    fn is_selectable(&self) -> bool {
        true
    }
}