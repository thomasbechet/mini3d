use mini3d_derive::{Resource, Serialize};

use crate::{input::provider::InputProviderHandle, utils::string::AsciiArray};

pub struct InputText {
    pub name: AsciiArray<32>,
    pub(crate) state: InputTextState,
}

#[derive(Serialize, Clone, Resource)]
pub struct InputTextState {
    pub value: String,
    pub(crate) handle: InputProviderHandle,
}
