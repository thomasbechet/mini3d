use mini3d_derive::{Reflect, Resource, Serialize};

use crate::{
    input::{provider::InputProviderHandle, MAX_INPUT_DISPLAY_NAME_LEN, MAX_INPUT_NAME_LEN},
    utils::{string::AsciiArray, uid::UID},
};

#[derive(Clone, Serialize, Resource, Reflect, Default)]
pub struct InputAction {
    pub name: AsciiArray<MAX_INPUT_NAME_LEN>,
    pub display_name: AsciiArray<MAX_INPUT_DISPLAY_NAME_LEN>,
    pub(crate) state: InputActionState,
}

impl InputAction {
    pub fn uid(&self) -> UID {
        UID::new(&self.name)
    }
}

#[derive(Serialize, Clone)]
pub struct InputActionState {
    pub(crate) pressed: bool,
    pub(crate) was_pressed: bool,
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
