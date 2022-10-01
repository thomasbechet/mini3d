use glam::Vec2;

use crate::input::{action::{ActionState, ActionInputId}, axis::AxisInputId};

pub struct ActionEvent {
    pub id: ActionInputId,
    pub state: ActionState,
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
    Action(ActionEvent),
    Axis(AxisEvent),
    Text(TextEvent),
}