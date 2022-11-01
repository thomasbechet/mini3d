use glam::Vec2;
use serde::{Serialize, Deserialize};

use crate::asset::{input_action::InputAction, input_axis::InputAxis, AssetRef};

#[derive(Serialize, Deserialize)]
pub struct InputActionEvent {
    pub action: AssetRef<InputAction>,
    pub pressed: bool,
}

#[derive(Serialize, Deserialize)]
pub struct InputAxisEvent {
    pub axis: AssetRef<InputAxis>,
    pub value: f32,
}

#[derive(Serialize, Deserialize)]
pub enum InputTextEvent {
    Character(char),
    String(String),
}

pub enum InputMouseEvent {
    Move { delta: Vec2 },
    Update { position: Vec2 },
}

pub enum InputEvent {
    Action(InputActionEvent),
    Axis(InputAxisEvent),
    Text(InputTextEvent),
}