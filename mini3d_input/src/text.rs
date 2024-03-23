use alloc::string::String;
use mini3d_derive::Serialize;
use mini3d_utils::{slot_map_key, string::AsciiArray};

use crate::provider::InputProviderHandle;

slot_map_key!(InputTextId);

#[derive(Clone, Serialize, Default)]
pub struct InputTextState {
    pub value: String,
}

#[derive(Serialize, Default)]
pub struct InputText {
    pub(crate) name: AsciiArray<32>,
    pub(crate) state: InputTextState,
    #[serialize(skip)]
    pub(crate) handle: InputProviderHandle,
}

impl InputText {
    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    pub fn value(&self) -> &str {
        self.state.value.as_str()
    }
}
