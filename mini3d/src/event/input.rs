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
pub struct InputTextEvent {
    pub stream: UID,
    pub value: String,
}

pub enum InputEvent {
    Action(InputActionEvent),
    Axis(InputAxisEvent),
    Text(InputTextEvent),
}