use glam::IVec2;
use mini3d_derive::{Component, Reflect};

use crate::{ecs::entity::Entity, math::rect::IRect, renderer::graphics::Graphics};

use self::layout::Navigation;

use super::ui_stylesheet::UIStyleSheet;

pub mod button;
pub mod checkbox;
pub mod label;
pub mod layout;
pub mod slider;
pub mod textbox;
pub mod viewport;

pub(crate) trait Widget {
    fn handle_event(&mut self, ctx: &mut EventContext, event: &Event) -> bool;
    fn render(&self, gfx: &mut Graphics, styles: &UIStyleSheet, offset: IVec2, time: f64);
    fn extent(&self) -> IRect;
    fn is_focusable(&self) -> bool;
    fn is_selectable(&self) -> bool;
}

pub(crate) enum UIWidgetKind {
    Button,
    CheckBox,
    Label,
    Slider,
    TextEdit,
}

#[derive(Default, Component, Reflect)]
pub(crate) struct UIWidget {
    kind: UIWidgetKind,
    z_index: i32,
    navigation: Navigation,
    next: Entity,
    prev: Entity,
}
