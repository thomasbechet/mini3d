use alloc::string::String;
use mini3d_db::slot_map_key_handle;
use mini3d_derive::Serialize;
use mini3d_utils::string::AsciiArray;

use crate::provider::InputProviderHandle;

slot_map_key_handle!(InputTextHandle);

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
