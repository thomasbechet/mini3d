use alloc::vec::Vec;
use mini3d_derive::{Reflect, Resource, Serialize};

use crate::{
    define_resource_handle, script::mir::primitive::PrimitiveType, utils::string::AsciiArray,
};

#[derive(Serialize, Reflect)]
pub struct StructField {
    name: AsciiArray<32>,
    ty: PrimitiveType,
}

#[derive(Default, Serialize, Reflect, Resource)]
pub struct StructDefinition {
    name: AsciiArray<32>,
    fields: Vec<StructField>,
}

define_resource_handle!(StructDefinitionHandle);
