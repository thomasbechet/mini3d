use glam::Vec2;

use crate::input::button::ButtonState;

pub struct ButtonEvent {
    pub name: &'static str,
    pub state: ButtonState,
}

pub struct AxisEvent {
    pub name: &'static str,
    pub value: f32,
}

pub enum TextEvent {
    Character(char),
    String(String),
}

pub enum CursorEvent {
    Move { delta: Vec2 },
    Update { position: Vec2 },
}

pub enum InputEvent {
    Button(ButtonEvent),
    Axis(AxisEvent),
    Text(TextEvent),
    Cursor(CursorEvent),
}