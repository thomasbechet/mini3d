use mini3d_derive::{Reflect, Resource, Serialize};

use crate::{
    input::{MAX_INPUT_DISPLAY_NAME_LEN, MAX_INPUT_NAME_LEN},
    utils::{string::AsciiArray, uid::UID},
};

#[derive(Default, Clone, Copy, Serialize)]
pub enum InputAxisRange {
    Clamped {
        min: f32,
        max: f32,
    },
    Normalized {
        norm: f32,
    },
    ClampedNormalized {
        min: f32,
        max: f32,
        norm: f32,
    },
    #[default]
    Infinite,
}

#[derive(Clone, Serialize, Resource, Reflect, Default)]
pub struct InputAxis {
    pub name: AsciiArray<MAX_INPUT_NAME_LEN>,
    pub display_name: AsciiArray<MAX_INPUT_DISPLAY_NAME_LEN>,
    pub range: InputAxisRange,
    pub default_value: f32,
}

impl InputAxis {
    pub fn uid(&self) -> UID {
        UID::new(&self.name)
    }
}
