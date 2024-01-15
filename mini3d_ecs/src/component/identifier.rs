use mini3d_derive::Serialize;
use mini3d_utils::string::AsciiArray;

#[derive(Default, Serialize)]
pub struct Identifier {
    pub(crate) name: AsciiArray<32>,
}

impl Identifier {
    pub const IDENT: &'static str = "identifier";

    pub fn new(name: &str) -> Self {
        Self {
            name: AsciiArray::from(name),
        }
    }
}
