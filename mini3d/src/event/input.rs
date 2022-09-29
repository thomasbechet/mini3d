use glam::Vec2;

use crate::input::{button::{ButtonState, ButtonInputId}, axis::AxisInputId};

pub struct ButtonEvent {
    pub id: ButtonInputId,
    pub state: ButtonState,
}

pub struct AxisEvent {
    pub id: AxisInputId,
    pub value: f32,
}

pub enum TextEvent {
    Character(char),
    String(String),
}

pub enum MouseEvent {
    Move { delta: Vec2 },
    Update { position: Vec2 },
}

pub enum InputEvent {
    Button(ButtonEvent),
    Axis(AxisEvent),
    Text(TextEvent),
    Mouse(MouseEvent),
}