use mini3d_db::slot_map_key_handle;
use mini3d_derive::Serialize;
use mini3d_math::fixed::I32F16;
use mini3d_utils::string::AsciiArray;

use crate::provider::InputProviderHandle;

slot_map_key_handle!(InputAxisHandle);

#[derive(Default, Clone, Copy, Serialize)]
pub enum InputAxisRange {
    Clamped {
        min: I32F16,
        max: I32F16,
    },
    Normalized {
        norm: I32F16,
    },
    ClampedNormalized {
        min: I32F16,
        max: I32F16,
        norm: I32F16,
    },
    #[default]
    Infinite,
}

#[derive(Clone, Default, Serialize)]
pub struct InputAxisState {
    pub(crate) value: I32F16,
}

#[derive(Clone, Default, Serialize)]
pub struct InputAxis {
    pub(crate) name: AsciiArray<64>,
    pub(crate) range: InputAxisRange,
    pub(crate) state: InputAxisState,
    #[serialize(skip)]
    pub(crate) handle: InputProviderHandle,
}

impl InputAxis {
    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    pub fn set_value(&mut self, value: I32F16) {
        self.state.value = match &self.range {
            InputAxisRange::Clamped { min, max } => value.max(*min).min(*max),
            InputAxisRange::Normalized { norm } => value / norm,
            InputAxisRange::ClampedNormalized { min, max, norm } => {
                value.max(*min).min(*max) / norm
            }
            InputAxisRange::Infinite => value,
        }
    }

    pub fn range(&self) -> InputAxisRange {
        self.range
    }

    pub fn value(&self) -> I32F16 {
        self.state.value
    }
}
