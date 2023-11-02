use mini3d_derive::{Reflect, Serialize};

use crate::{
    feature::core::resource::{ResourceData, ResourceHookContext},
    input::{provider::InputProviderHandle, MAX_INPUT_DISPLAY_NAME_LEN, MAX_INPUT_NAME_LEN},
    resource::handle::ResourceHandle,
    utils::{string::AsciiArray, uid::UID},
};

#[derive(Clone, Reflect, Default, Serialize)]
pub struct InputAction {
    pub name: AsciiArray<MAX_INPUT_NAME_LEN>,
    pub display_name: AsciiArray<MAX_INPUT_DISPLAY_NAME_LEN>,
    pub(crate) state: InputActionState,
}

impl ResourceData for InputAction {
    fn hook_added(handle: ResourceHandle, ctx: ResourceHookContext) {
        ctx.input.on_action_added(handle, ctx.resource);
    }

    fn hook_removed(handle: ResourceHandle, ctx: ResourceHookContext) {
        ctx.input.on_action_removed(handle, ctx.resource);
    }
}

impl InputAction {
    pub fn uid(&self) -> UID {
        UID::new(&self.name)
    }
}

#[derive(Clone, Serialize)]
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