use mini3d_derive::Serialize;

use crate::utils::{slotmap::SlotId, uid::UID};

#[derive(Default, Serialize, Clone, Copy)]
pub struct InputActionHandle {
    #[serialize(skip)]
    pub(crate) id: SlotId,
    pub(crate) uid: UID,
}

impl InputActionHandle {
    pub fn resolve(&mut self, resolver: &mut ReferenceResolver) {}
}

#[derive(Default, Serialize, Clone, Copy)]
pub struct InputAxisHandle {
    #[serialize(skip)]
    pub(crate) id: SlotId,
    pub(crate) uid: UID,
}

impl InputAxisHandle {
    pub fn resolve(&mut self, resolver: &mut ReferenceResolver) {}
}

#[derive(Default, Serialize, Clone, Copy)]
pub struct InputTextHandle {
    #[serialize(skip)]
    pub(crate) id: SlotId,
    pub(crate) uid: UID,
}

impl InputTextHandle {
    pub fn resolve(&mut self, resolver: &mut ReferenceResolver) {}
}
