use glam::IVec2;
use mini3d_derive::Serialize;

use crate::{
    feature::ui::ui_stylesheet::UIStyleSheet,
    math::rect::IRect,
    renderer::{color::Color, graphics::Graphics},
    ui::{
        event::{Event, EventContext, UIEvent},
        style::UIBoxStyle,
    },
    utils::uid::UID,
};

use super::Widget;

#[derive(Serialize, Clone)]
pub struct UICheckBoxStyle {
    checked: UIBoxStyle,
    unchecked: UIBoxStyle,
}

impl UICheckBoxStyle {
    pub const DEFAULT: &'static str = "default";

    pub fn new(checked: UIBoxStyle, unchecked: UIBoxStyle) -> Self {
        Self { checked, unchecked }
    }
}

impl Default for UICheckBoxStyle {
    fn default() -> Self {
        Self {
            checked: UIBoxStyle::Color(Color::WHITE),
            unchecked: UIBoxStyle::Color(Color::RED),
        }
    }
}

#[derive(Serialize, Clone)]
pub struct UICheckBox {
    extent: IRect,
    checked: bool,
    hovered: bool,
    style: UID,
    on_checked: Option<UID>,
    on_unchecked: Option<UID>,
}

impl UICheckBox {
    pub fn new(extent: IRect, checked: bool) -> Self {
        Self {
            extent,
            checked,
            style: UICheckBoxStyle::DEFAULT.into(),
            hovered: false,
            on_checked: None,
            on_unchecked: None,
        }
    }

    pub fn on_checked(&mut self, action: Option<UID>) {
        self.on_checked = action;
    }

    pub fn on_unchecked(&mut self, action: Option<UID>) {
        self.on_unchecked = action;
    }
}

impl Widget for UICheckBox {
    fn handle_event(&mut self, ctx: &mut EventContext, event: &Event) -> bool {
        match event {
            Event::PrimaryJustPressed => {
                self.checked = !self.checked;
                if self.checked {
                    if let Some(action) = self.on_checked {
                        ctx.events.push(UIEvent::Action {
                            user: ctx.user.uid(),
                            id: action,
                        });
                    }
                } else if let Some(action) = self.on_unchecked {
                    ctx.events.push(UIEvent::Action {
                        user: ctx.user.uid(),
                        id: action,
                    });
                }
            }
            Event::Enter => {
                self.hovered = true;
            }
            Event::Leave => {
                self.hovered = false;
            }
            _ => {}
        }
        true
    }

    fn render(&self, gfx: &mut Graphics, styles: &UIStyleSheet, offset: IVec2, _time: f64) {
        if let Some(style) = styles.checkboxes.get(&self.style) {
            let extent = self.extent.translate(offset);
            if self.checked {
                style.checked.render(gfx, extent, Color::WHITE, 1);
            } else {
                style.unchecked.render(gfx, extent, Color::WHITE, 1);
            }
        }
    }

    fn extent(&self) -> IRect {
        self.extent
    }

    fn is_focusable(&self) -> bool {
        false
    }

    fn is_selectable(&self) -> bool {
        true
    }
}
