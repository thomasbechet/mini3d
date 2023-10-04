use mini3d_derive::{Reflect, Resource, Serialize};

use crate::{
    input::{MAX_INPUT_DISPLAY_NAME_LEN, MAX_INPUT_NAME_LEN},
    utils::{string::AsciiArray, uid::UID},
};

#[derive(Clone, Serialize, Resource, Reflect, Default)]
pub struct InputAction {
    pub name: AsciiArray<MAX_INPUT_NAME_LEN>,
    pub display_name: AsciiArray<MAX_INPUT_DISPLAY_NAME_LEN>,
    pub default_pressed: bool,
}

impl InputAction {
    pub fn uid(&self) -> UID {
        UID::new(&self.name)
    }
}
