use alloc::string::String;
use mini3d_derive::Serialize;

use crate::math::fixed::I32F16;

use super::resource::{InputActionHandle, InputAxisHandle, InputTextHandle};

#[derive(Serialize)]
pub struct InputActionEvent {
    pub handle: InputActionHandle,
    pub pressed: bool,
}

#[derive(Serialize)]
pub struct InputAxisEvent {
    pub handle: InputAxisHandle,
    pub value: I32F16,
}

#[derive(Serialize)]
pub struct InputTextEvent {
    pub handle: InputTextHandle,
    pub value: String,
}

pub enum InputEvent {
    Action(InputActionEvent),
    Axis(InputAxisEvent),
    Text(InputTextEvent),
}
