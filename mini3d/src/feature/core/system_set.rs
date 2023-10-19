use crate::{resource::handle::ResourceRef, utils::string::AsciiArray};

#[derive(Default)]
pub struct SystemOrder;

pub struct SystemInstance {
    name: AsciiArray<32>,
    system: ResourceRef,
    stage: ResourceRef,
    order: SystemOrder,
}

pub struct SystemSet(Vec<SystemInstance>);
