use alloc::vec::Vec;
use mini3d_derive::{Reflect, Serialize};

use crate::{script::mir::primitive::PrimitiveType, utils::string::AsciiArray};

#[derive(Serialize, Reflect)]
pub struct StructField {
    name: AsciiArray<32>,
    ty: PrimitiveType,
}

#[derive(Default, Serialize, Reflect)]
pub struct StructDefinition {
    name: AsciiArray<32>,
    fields: Vec<StructField>,
}
