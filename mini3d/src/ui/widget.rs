use anyhow::Result;
use glam::IVec2;

use crate::{renderer::graphics::Graphics, math::rect::IRect, feature::asset::ui_stylesheet::UIStyleSheet};

use super::event::{EventContext, Event};

pub mod button;
pub mod checkbox;
pub mod label;
pub mod layout;
pub mod slider;
pub mod sprite;
pub mod textbox;
pub mod viewport;

pub(crate) trait Widget {
    fn handle_event(&mut self, ctx: &mut EventContext, event: &Event) -> bool;
    fn render(&self, gfx: &mut Graphics, styles: &UIStyleSheet, offset: IVec2, time: f64) -> Result<()>;
    fn extent(&self) -> IRect;
    fn is_focusable(&self) -> bool;
    fn is_selectable(&self) -> bool;
}