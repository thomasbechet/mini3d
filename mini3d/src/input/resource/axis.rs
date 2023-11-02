use mini3d_derive::{Reflect, Serialize};

use crate::{
    feature::core::resource::{ResourceData, ResourceHookContext},
    input::{provider::InputProviderHandle, MAX_INPUT_DISPLAY_NAME_LEN},
    resource::handle::ResourceHandle,
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

#[derive(Clone, Reflect, Default, Serialize)]
pub struct InputAxis {
    pub display_name: AsciiArray<MAX_INPUT_DISPLAY_NAME_LEN>,
    pub range: InputAxisRange,
    pub(crate) state: InputAxisState,
}

impl ResourceData for InputAxis {
    fn hook_added(handle: ResourceHandle, ctx: ResourceHookContext) {
        ctx.input.on_axis_added(handle, ctx.resource);
    }

    fn hook_removed(handle: ResourceHandle, ctx: ResourceHookContext) {
        ctx.input.on_axis_removed(handle, ctx.resource);
    }
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

#[derive(Clone, Reflect, Default, Serialize)]
pub struct InputAxisState {
    pub value: f32,
    #[serialize(skip)]
    pub(crate) handle: InputProviderHandle,
}