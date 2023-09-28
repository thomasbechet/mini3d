use mini3d_derive::Serialize;

use crate::{
    registry::datatype::{ReferenceError, ReferenceResolver},
    utils::{slotmap::SlotId, uid::UID},
};

#[derive(Default, Serialize, Hash, PartialEq, Eq, Clone, Copy, Debug)]
pub struct AssetRef {
    #[serialize(skip)]
    entry: SlotId,
    key: UID,
}

impl AssetRef {
    pub fn resolve(&mut self, resolver: &mut ReferenceResolver) -> Result<(), ReferenceError> {
        Ok(())
    }
}
