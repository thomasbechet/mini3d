use mini3d_derive::Serialize;
use mini3d_utils::string::AsciiArray;

use super::{Component, ComponentStorage};

#[derive(Default, Serialize)]
pub struct Identifier {
    pub(crate) name: AsciiArray<32>,
}

impl Identifier {
    pub fn new(name: &str) -> Self {
        Self {
            name: AsciiArray::from(name),
        }
    }
}

impl Component for Identifier {
    const NAME: &'static str = "identifier";
    const STORAGE: ComponentStorage = ComponentStorage::Single;
}
