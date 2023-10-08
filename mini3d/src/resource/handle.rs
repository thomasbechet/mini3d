use std::fmt::Debug;

use mini3d_derive::Serialize;

use crate::{
    registry::datatype::ReferenceResolver,
    utils::{slotmap::SlotId, uid::UID},
};

#[derive(Default, Serialize, Hash, PartialEq, Eq, Clone, Copy, Debug)]
pub struct ResourceHandle {
    #[serialize(skip)]
    pub(crate) id: SlotId,
    pub(crate) uid: UID,
}

impl ResourceHandle {
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
