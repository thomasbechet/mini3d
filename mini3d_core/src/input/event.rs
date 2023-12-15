use mini3d_derive::Serialize;

use crate::{math::fixed::I32F16, utils::string::AsciiArray};

#[derive(Serialize)]
pub struct InputActionEvent {
    pub id: u32,
    pub pressed: bool,
}

#[derive(Serialize)]
pub struct InputAxisEvent {
    pub id: u32,
    pub value: I32F16,
}

#[derive(Serialize)]
pub struct InputTextEvent {
    pub id: u32,
    pub value: AsciiArray<64>,
}

pub enum InputEvent {
    Action(InputActionEvent),
    Axis(InputAxisEvent),
    Text(InputTextEvent),
}
