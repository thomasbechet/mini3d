use alloc::string::String;
use mini3d_math::fixed::I32F16;

use crate::{action::InputActionHandle, axis::InputAxisHandle, text::InputTextId};

pub struct InputActionEvent {
    pub action: InputActionHandle,
    pub pressed: bool,
}

pub struct InputAxisEvent {
    pub axis: InputAxisHandle,
    pub value: I32F16,
}

pub struct InputTextEvent {
    pub text: InputTextId,
    pub value: String,
}

pub enum InputEvent {
    Action(InputActionEvent),
    Axis(InputAxisEvent),
    Text(InputTextEvent),
}
