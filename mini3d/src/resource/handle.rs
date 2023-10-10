use std::fmt::Debug;

use mini3d_derive::Serialize;

use crate::utils::{slotmap::SlotId, uid::UID};

pub struct ReferenceResolver;

#[derive(Default, Hash, PartialEq, Eq, Clone, Copy, Debug)]
pub struct ResourceHandle(pub(crate) SlotId);

#[derive(Default, Serialize)]
pub struct ResourceRef {
    #[serialize(skip)]
    pub(crate) id: SlotId,
    pub(crate) uid: UID,
}

impl ResourceRef {
    pub fn resolve(&mut self, resolver: &mut ReferenceResolver) {
        if !self.uid.is_null() {
            if self.id.is_null() {
                // Find entry
                self.id = resolver.resolve_resource_id(self.uid);
            } else {
                // The entry's key has changed
                self.uid = resolver.remap_resource_key(self.id);
            }
        }
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

impl ToResourceHandle for ResourceRef {
    fn to_handle(&self) -> ResourceHandle {
        ResourceHandle(self.id)
    }
}
