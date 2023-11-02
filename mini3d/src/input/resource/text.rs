use mini3d_derive::{Reflect, Resource, Serialize};

use crate::{input::provider::InputProviderHandle, utils::string::AsciiArray};

pub struct InputText {
    pub name: AsciiArray<32>,
    pub(crate) state: InputTextState,
}

#[derive(Clone, Resource, Serialize, Reflect, Default)]
pub struct InputTextState {
    pub value: String,
    #[serialize(skip)]
    pub(crate) handle: InputProviderHandle,
}
