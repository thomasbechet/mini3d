use mini3d_derive::{Reflect, Serialize};

use crate::{
    define_resource_handle,
    feature::core::resource::{Resource, ResourceHookContext},
    input::provider::InputProviderHandle,
    math::fixed::I32F16,
    resource::handle::ResourceHandle,
    utils::string::AsciiArray,
};

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

#[derive(Clone, Reflect, Default, Serialize)]
pub struct InputAxis {
    pub display_name: AsciiArray<64>,
    pub range: InputAxisRange,
    pub(crate) state: InputAxisState,
}

impl InputAxis {
    pub const NAME: &'static str = "RTY_InputAxis";
}

impl Resource for InputAxis {
    fn hook_added(handle: ResourceHandle, ctx: ResourceHookContext) {
        ctx.input.on_axis_added(handle.into(), ctx.resource);
    }

    fn hook_removed(handle: ResourceHandle, ctx: ResourceHookContext) {
        ctx.input.on_axis_removed(handle.into(), ctx.resource);
    }
}

impl InputAxis {
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
}

#[derive(Clone, Reflect, Default, Serialize)]
pub struct InputAxisState {
    pub value: I32F16,
    #[serialize(skip)]
    pub(crate) handle: InputProviderHandle,
}

define_resource_handle!(InputAxisHandle);
