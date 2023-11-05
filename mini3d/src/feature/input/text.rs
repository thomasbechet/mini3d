use mini3d_derive::{Reflect, Resource, Serialize};

use crate::{
    define_resource_handle, feature::core::resource::Resource,
    input::provider::InputProviderHandle, utils::string::AsciiArray,
};

#[derive(Serialize, Default, Reflect)]
pub struct InputText {
    pub name: AsciiArray<32>,
    pub(crate) state: InputTextState,
}

impl InputText {
    pub const NAME: &'static str = "RTY_InputText";
}

impl Resource for InputText {}

#[derive(Clone, Resource, Serialize, Reflect, Default)]
pub struct InputTextState {
    pub value: String,
    #[serialize(skip)]
    pub(crate) handle: InputProviderHandle,
}

define_resource_handle!(InputTextHandle);
