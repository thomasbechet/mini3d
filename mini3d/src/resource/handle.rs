use std::fmt::Debug;

use mini3d_derive::Serialize;

use crate::{ecs::entity::Entity, utils::slotmap::SlotId};

use super::ResourceManager;

pub struct ReferenceResolver;

impl ReferenceResolver {
    pub(crate) fn resolve_resource<H: ToResourceHandle + Default>(
        &mut self,
        handle: impl ToResourceHandle,
    ) -> H {
        Default::default()
    }
    pub(crate) fn resolve_entity(&mut self, entity: Entity) -> Entity {
        Default::default()
    }
}

#[derive(Default, Hash, PartialEq, Eq, Clone, Copy, Debug, Serialize)]
pub struct ResourceHandle(pub(crate) SlotId);

impl ResourceHandle {
    pub fn null() -> Self {
        Self(SlotId::null())
    }

    pub fn is_null(&self) -> bool {
        self.0.is_null()
    }

    pub fn resolve(&mut self, resolver: &mut ReferenceResolver) {
        if !self.0.is_null() {
            *self = resolver.resolve_resource(*self);
        }
    }

    pub(crate) fn release(&mut self, resources: &mut ResourceManager) {
        if !self.0.is_null() {
            resources.decrement_ref(*self);
            self.0 = SlotId::null();
        }
    }

    pub(crate) fn from_raw(raw: u32) -> Self {
        Self(SlotId::from_raw(raw))
    }
}

pub trait ToResourceHandle {
    fn to_handle(&self) -> ResourceHandle;
    fn from_handle(handle: ResourceHandle) -> Self;
}

impl ToResourceHandle for ResourceHandle {
    fn to_handle(&self) -> ResourceHandle {
        *self
    }
    fn from_handle(handle: ResourceHandle) -> Self {
        handle
    }
}
