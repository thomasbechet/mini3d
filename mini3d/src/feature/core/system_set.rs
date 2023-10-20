use crate::{
    resource::handle::{ReferenceResolver, ResourceRef},
    utils::string::AsciiArray,
};

use super::resource_type::Resource;

#[derive(Default)]
pub struct SystemOrder;

pub struct SystemInstance {
    name: AsciiArray<32>,
    system: ResourceRef,
    stage: ResourceRef,
    order: SystemOrder,
}

pub struct SystemSet(Vec<SystemInstance>);

impl Resource for SystemSet {
    fn resolve_references(&mut self, resolver: &mut ReferenceResolver) {
        for system in self.0.iter_mut() {
            system.system.resolve(resolver);
            system.stage.resolve(resolver);
        }
    }
}
