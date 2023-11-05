use mini3d_derive::{Reflect, Serialize};

use crate::{
    define_resource_handle,
    feature::core::resource::{Resource, ResourceHookContext},
    input::provider::InputProviderHandle,
    resource::handle::ResourceHandle,
    utils::string::AsciiArray,
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

#[derive(Clone, Reflect, Default, Serialize)]
pub struct InputAxisState {
    pub value: f32,
    #[serialize(skip)]
    pub(crate) handle: InputProviderHandle,
}

define_resource_handle!(InputAxisHandle);
