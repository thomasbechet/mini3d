use crate::{script::mir::primitive::PrimitiveType, utils::string::AsciiArray};

pub struct StructField {
    name: AsciiArray<32>,
    ty: PrimitiveType,
}

pub struct StructDefinition {
    name: AsciiArray<32>,
    fields: Vec<StructField>,
}

impl StructDefinition {
    pub fn size(&self) -> usize {}
}
