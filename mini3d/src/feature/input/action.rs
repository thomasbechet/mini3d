use mini3d_derive::{Reflect, Serialize};

use crate::{
    define_resource_handle,
    feature::core::resource::{Resource, ResourceHookContext},
    input::provider::InputProviderHandle,
    resource::handle::ResourceHandle,
    utils::string::AsciiArray,
};

#[derive(Clone, Reflect, Default, Serialize)]
pub struct InputAction {
    pub display_name: AsciiArray<64>,
    pub(crate) state: InputActionState,
}

impl InputAction {
    pub const NAME: &'static str = "RTY_InputAction";
}

impl Resource for InputAction {
    fn hook_added(handle: ResourceHandle, ctx: ResourceHookContext) {
        ctx.input.on_action_added(handle.into(), ctx.resource);
    }

    fn hook_removed(handle: ResourceHandle, ctx: ResourceHookContext) {
        ctx.input.on_action_removed(handle.into(), ctx.resource);
    }
}

#[derive(Default, Clone, Serialize)]
pub struct InputActionState {
    pub(crate) pressed: bool,
    pub(crate) was_pressed: bool,
    #[serialize(skip)]
    pub(crate) handle: InputProviderHandle,
}

impl InputActionState {
    pub fn is_pressed(&self) -> bool {
        self.pressed
    }

    pub fn is_released(&self) -> bool {
        !self.pressed
    }

    pub fn is_just_pressed(&self) -> bool {
        self.pressed && !self.was_pressed
    }

    pub fn is_just_released(&self) -> bool {
        !self.pressed && self.was_pressed
    }
}

define_resource_handle!(InputActionHandle);
