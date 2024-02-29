use alloc::string::String;
use mini3d_math::fixed::I32F16;

use crate::{action::InputActionId, axis::InputAxisId, text::InputTextHandle};

pub struct InputActionEvent {
    pub action: InputActionId,
    pub pressed: bool,
}

pub struct InputAxisEvent {
    pub axis: InputAxisId,
    pub value: I32F16,
}

pub struct InputTextEvent {
    pub text: InputTextHandle,
    pub value: String,
}

pub enum InputEvent {
    Action(InputActionEvent),
    Axis(InputAxisEvent),
    Text(InputTextEvent),
}
