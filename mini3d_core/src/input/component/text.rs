use alloc::string::String;
use mini3d_derive::{Reflect, Serialize};

use crate::{
    ecs::component::{Component, ComponentStorage},
    input::provider::InputProviderHandle,
    utils::string::AsciiArray,
};

#[derive(Clone, Serialize, Reflect, Default)]
pub struct InputTextState {
    pub value: String,
}

#[derive(Serialize, Default, Reflect)]
pub struct InputText {
    pub(crate) name: AsciiArray<32>,
    pub(crate) state: InputTextState,
    #[serialize(skip)]
    pub(crate) handle: InputProviderHandle,
}

impl InputText {
    pub const NAME: &'static str = "RTY_InputText";

    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    pub fn value(&self) -> &str {
        self.state.value.as_str()
    }
}

impl Component for InputText {
    const STORAGE: ComponentStorage = ComponentStorage::Single;
}
