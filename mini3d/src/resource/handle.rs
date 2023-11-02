use std::fmt::Debug;

use mini3d_derive::Serialize;

use crate::{
    define_resource_handle,
    ecs::entity::Entity,
    serialize::{Decoder, Encoder, Serialize},
    utils::slotmap::SlotId,
};

use super::ResourceManager;

pub struct ReferenceResolver;

impl ReferenceResolver {
    pub(crate) fn resolve_resource(&mut self, handle: ResourceHandle) -> ResourceHandle {}
    pub(crate) fn resolve_entity(&mut self, entity: Entity) -> Entity {}
}

#[derive(Default, Hash, PartialEq, Eq, Clone, Copy, Debug, Serialize)]
pub struct ResourceHandle(pub(crate) SlotId);

impl ResourceHandle {
    pub fn resolve(&mut self, resolver: &mut ReferenceResolver) {
        if !self.0.is_null() {
            self.0 = resolver.resolve_resource(self.0);
        }
    }

    pub fn handle(&self) -> ResourceHandle {
        ResourceHandle(self.0)
    }

    pub(crate) fn release(&mut self, resources: &mut ResourceManager) {
        if !self.0.is_null() {
            resources.decrement_ref(self.0);
            self.0 = ResourceHandle::null();
        }
    }

    pub(crate) fn from_raw(raw: u32) -> Self {
        Self(SlotId::from_raw(raw))
    }
}

pub trait ToResourceHandle {
    fn to_handle(&self) -> ResourceHandle;
}

impl ToResourceHandle for ResourceHandle {
    fn to_handle(&self) -> ResourceHandle {
        *self
    }
}

define_resource_handle!(ResourceTypeHandle);
