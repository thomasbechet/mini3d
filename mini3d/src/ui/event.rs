use glam::IVec2;
use serde::{Serialize, Deserialize};

use crate::uid::UID;

use super::user::UIUser;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Direction {
    Up = 0,
    Down = 1,
    Left = 2,
    Right = 3,
}

#[derive(Debug, Clone)]
pub enum UIEvent {
    Action { user: UID, id: UID },
    FloatChanged { user: UID, id: UID, value: f32 },
}

#[derive(Serialize, Deserialize)]
pub(crate) enum Event {
    PrimaryJustPressed,
    PrimaryJustReleased,
    Cancel,
    Enter,
    Leave,
    GainFocus,
    LooseFocus,
    Text { value: String },
    Scroll { value: f32 },
    SelectionMoved { direction: Direction },
    CursorMoved { position: IVec2 },
    ModeChanged,
}

pub(crate) struct EventContext<'a> {
    pub(crate) user: &'a mut UIUser,
    pub(crate) events: &'a mut Vec<UIEvent>,
    pub(crate) time: f64,
}