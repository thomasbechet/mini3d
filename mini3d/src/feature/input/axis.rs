use mini3d_derive::{Reflect, Resource, Serialize};

use crate::{
    input::{provider::InputProviderHandle, MAX_INPUT_DISPLAY_NAME_LEN, MAX_INPUT_NAME_LEN},
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
    pub(crate) state: InputAxisState,
}

impl InputAxis {
    pub fn set_value(&mut self, value: f32) {
        self.state.value = match &self.range {
            InputAxisRange::Clamped { min, max } => value.max(*min).min(*max),
            InputAxisRange::Normalized { norm } => value / norm,
            InputAxisRange::ClampedNormalized { min, max, norm } => {
                value.max(*min).min(*max) / norm
            }
            InputAxisRange::Infinite => value,
        }
    }
}

impl InputAxis {
    pub fn uid(&self) -> UID {
        UID::new(&self.name)
    }
}

#[derive(Serialize, Clone, Resource)]
pub struct InputAxisState {
    pub value: f32,
    pub(crate) handle: InputProviderHandle,
}
