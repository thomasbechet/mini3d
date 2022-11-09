use glam::Vec2;
use serde::{Serialize, Deserialize};

use crate::uid::UID;

#[derive(Serialize, Deserialize)]
pub struct InputActionEvent {
    pub action: UID,
    pub pressed: bool,
}

#[derive(Serialize, Deserialize)]
pub struct InputAxisEvent {
    pub axis: UID,
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