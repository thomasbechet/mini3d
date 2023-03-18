use crate::{renderer::graphics::Graphics, math::rect::IRect};

use super::event::{EventContext, Event};

pub mod button;
pub mod checkbox;
pub mod graphics;
pub mod label;
pub mod layout;
pub mod slider;
pub mod sprite;
pub mod textbox;
pub mod viewport;

pub(crate) trait Widget {
    fn handle_event(&mut self, ctx: &mut EventContext, event: &Event) -> bool;
    fn render(&self, gfx: &mut Graphics, time: f64);
    fn extent(&self) -> IRect;
    fn is_focusable(&self) -> bool;
}