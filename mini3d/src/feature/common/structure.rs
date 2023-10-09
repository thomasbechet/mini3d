use crate::{script::mir::primitive::PrimitiveType, utils::string::AsciiArray};

pub const MAX_STRUCT_FIELD_NAME_LEN: usize = 32;
pub const MAX_STRUCT_NAME_LEN: usize = 32;

pub struct StructField {
    name: AsciiArray<MAX_STRUCT_FIELD_NAME_LEN>,
    ty: PrimitiveType,
}

pub struct StructDefinition {
    name: AsciiArray<MAX_STRUCT_NAME_LEN>,
    fields: Vec<StructField>,
}

impl StructDefinition {
    pub fn size(&self) -> usize {}
}
