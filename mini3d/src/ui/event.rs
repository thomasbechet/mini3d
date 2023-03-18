use glam::IVec2;

use crate::uid::UID;

use super::profile::Profile;

#[derive(Debug, Clone, Copy)]
pub(crate) enum Direction {
    Up = 0,
    Down = 1,
    Left = 2,
    Right = 3,
}

impl Direction {
    pub(crate) const COUNT: usize = 4;
}

pub enum UIEvent {
    ButtonClicked {
        id: UID,
        profile: UID,
    },
    CheckboxChanged {
        id: UID,
        profile: UID,
        checked: bool,
    },
    SliderValueChanged {
        id: UID,
        profile: UID,
        value: f32,
    },
}

pub(crate) enum Event {
    PrimaryJustPressed,
    PrimaryJustReleased,
    SecondaryJustPressed,
    SecondaryJustReleased,
    Enter,
    Leave,
    GainFocus,
    LooseFocus,
    Text { value: String },
    Scroll { value: f32 },
    SelectionMove { direction: Direction },
    CursorMove { position: IVec2 },
    ModeChange,
}

pub(crate) struct EventContext<'a> {
    pub(crate) profile: &'a mut Profile,
    pub(crate) events: &'a mut Vec<UIEvent>,
    pub(crate) time: f64,
}