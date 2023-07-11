use glam::IVec2;
use mini3d_derive::Serialize;

use crate::{
    feature::component::ui::ui_stylesheet::UIStyleSheet,
    math::rect::IRect,
    renderer::{color::Color, graphics::Graphics},
    ui::{
        event::{Event, EventContext, UIEvent},
        style::UIBoxStyle,
    },
    uid::UID,
};

use super::Widget;

#[derive(Serialize, Clone)]
pub struct UIButtonStyle {
    normal: UIBoxStyle,
    pressed: UIBoxStyle,
    hovered: UIBoxStyle,
}

impl UIButtonStyle {
    pub const DEFAULT: &'static str = "default";

    pub fn new(normal: UIBoxStyle, pressed: UIBoxStyle, hovered: UIBoxStyle) -> Self {
        Self {
            normal,
            pressed,
            hovered,
        }
    }
}

impl Default for UIButtonStyle {
    fn default() -> Self {
        Self {
            normal: UIBoxStyle::Color(Color::WHITE),
            pressed: UIBoxStyle::Color(Color::RED),
            hovered: UIBoxStyle::Color(Color::GRAY),
        }
    }
}

#[derive(Serialize)]
pub struct UIButton {
    pressed: bool,
    hovered: bool,
    extent: IRect,
    style: UID,
    on_pressed: Option<UID>,
    on_released: Option<UID>,
}

impl UIButton {
    pub fn new(extent: IRect) -> Self {
        Self {
            extent,
            style: UIButtonStyle::DEFAULT.into(),
            pressed: false,
            hovered: false,
            on_pressed: None,
            on_released: None,
        }
    }

    pub fn on_pressed(&mut self, action: Option<UID>) {
        self.on_pressed = action;
    }

    pub fn on_released(&mut self, action: Option<UID>) {
        self.on_released = action;
    }
}

impl Widget for UIButton {
    fn handle_event(&mut self, ctx: &mut EventContext, event: &Event) -> bool {
        match event {
            Event::PrimaryJustPressed => {
                if !self.pressed {
                    self.pressed = true;
                    if let Some(action) = self.on_pressed {
                        ctx.events.push(UIEvent::Action {
                            user: ctx.user.uid(),
                            id: action,
                        });
                    }
                }
            }
            Event::PrimaryJustReleased => {
                if self.pressed {
                    self.pressed = false;
                    if let Some(action) = self.on_released {
                        ctx.events.push(UIEvent::Action {
                            user: ctx.user.uid(),
                            id: action,
                        });
                    }
                }
            }
            Event::Enter => {
                self.hovered = true;
            }
            Event::Leave => {
                self.hovered = false;
                if self.pressed {
                    if let Some(action) = self.on_released {
                        ctx.events.push(UIEvent::Action {
                            user: ctx.user.uid(),
                            id: action,
                        });
                    }
                    self.pressed = false;
                }
            }
            _ => {}
        }
        true
    }

    fn render(&self, gfx: &mut Graphics, styles: &UIStyleSheet, offset: IVec2, _time: f64) {
        if let Some(style) = styles.buttons.get(&self.style) {
            let extent = self.extent.translate(offset);
            if self.pressed {
                style.pressed.render(gfx, extent, Color::WHITE, 1);
            } else if self.hovered {
                style.hovered.render(gfx, extent, Color::WHITE, 1);
            } else {
                style.normal.render(gfx, extent, Color::WHITE, 1);
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
