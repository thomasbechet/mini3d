use mini3d_derive::{Reflect, Resource, Serialize};

use crate::{script::mir::primitive::PrimitiveType, utils::string::AsciiArray};

#[derive(Serialize, Reflect, Default)]
pub struct StructField {
    name: AsciiArray<32>,
    ty: PrimitiveType,
}

#[derive(Serialize, Reflect, Default, Resource)]
pub struct StructDefinition {
    name: AsciiArray<32>,
    fields: Vec<StructField>,
}

impl StructDefinition {
    pub fn size(&self) -> usize {}
}
