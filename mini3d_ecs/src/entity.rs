use alloc::vec::Vec;
use mini3d_derive::Serialize;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize)]
pub struct Entity(pub(crate) u32);

pub(crate) type EntityVersion = u8;
pub(crate) type EntityIndex = u16;

impl Entity {
    pub(crate) fn new(index: EntityIndex, version: EntityVersion) -> Self {
        Self(index as u32 | ((version as u32) << 24))
    }

    pub(crate) fn index(&self) -> EntityIndex {
        (self.0 & 0xffff) as EntityIndex
    }

    pub(crate) fn version(&self) -> EntityVersion {
        (self.0 >> 24) as EntityVersion
    }

    pub fn null() -> Self {
        Self(0)
    }

    pub fn raw(&self) -> u32 {
        self.0
    }
}

impl Default for Entity {
    fn default() -> Self {
        Self::null()
    }
}

#[derive(Default)]
pub(crate) struct EntityTable {
    free_entities: Vec<Entity>,
}
