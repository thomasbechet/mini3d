use mini3d_derive::Serialize;
use mini3d_utils::{slot_map_key, string::AsciiArray};

use crate::provider::InputProviderHandle;

slot_map_key!(InputActionHandle);

#[derive(Default, Clone, Serialize)]
pub struct InputActionState {
    pub(crate) pressed: bool,
    pub(crate) was_pressed: bool,
}

#[derive(Clone, Default, Serialize)]
pub struct InputAction {
    pub(crate) name: AsciiArray<32>,
    pub(crate) state: InputActionState,
    #[serialize(skip)]
    pub(crate) handle: InputProviderHandle,
}

impl InputAction {
    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    pub fn is_pressed(&self) -> bool {
        self.state.pressed
    }

    pub fn is_released(&self) -> bool {
        !self.state.pressed
    }

    pub fn is_just_pressed(&self) -> bool {
        self.state.pressed && !self.state.was_pressed
    }

    pub fn is_just_released(&self) -> bool {
        !self.state.pressed && self.state.was_pressed
    }
}
