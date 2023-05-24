use mini3d_derive::Serialize;

use crate::uid::UID;

#[derive(Serialize)]
pub struct InputActionEvent {
    pub action: UID,
    pub pressed: bool,
}

#[derive(Serialize)]
pub struct InputAxisEvent {
    pub axis: UID,
    pub value: f32,
}

#[derive(Serialize)]
pub struct InputTextEvent {
    pub stream: UID,
    pub value: String,
}

pub enum InputEvent {
    Action(InputActionEvent),
    Axis(InputAxisEvent),
    Text(InputTextEvent),
}