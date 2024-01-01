use alloc::string::String;
use mini3d_derive::Serialize;

use crate::{ecs::entity::Entity, math::fixed::I32F16};

#[derive(Serialize)]
pub struct InputActionEvent {
    pub action: Entity,
    pub pressed: bool,
}

#[derive(Serialize)]
pub struct InputAxisEvent {
    pub axis: Entity,
    pub value: I32F16,
}

#[derive(Serialize)]
pub struct InputTextEvent {
    pub text: Entity,
    pub value: String,
}

pub enum InputEvent {
    Action(InputActionEvent),
    Axis(InputAxisEvent),
    Text(InputTextEvent),
}
