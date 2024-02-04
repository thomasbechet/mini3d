use alloc::vec::Vec;
use mini3d_derive::Serialize;

use crate::bitset::Bitset;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize)]
pub struct Entity(pub(crate) u32);

pub(crate) type EntityVersion = u8;
pub(crate) type EntityIndex = u16;

impl Entity {
    pub(crate) fn new(index: EntityIndex, version: EntityVersion) -> Self {
        Self(index as u32 | ((version as u32) << 16))
    }

    pub(crate) fn index(&self) -> EntityIndex {
        (self.0 & 0xffff) as EntityIndex
    }

    pub(crate) fn version(&self) -> EntityVersion {
        (self.0 >> 16) as EntityVersion
    }

    pub fn null() -> Self {
        Self(!0)
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

impl core::fmt::Display for Entity {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:06X}", self.0)
    }
}

#[derive(Default)]
pub(crate) struct EntityTable {
    free_entities: Vec<Entity>,
    next_index: EntityIndex, // Default index is 0
    versions: Vec<EntityVersion>,
    bitset: Bitset,
}

impl EntityTable {
    pub(crate) fn create(&mut self) -> Entity {
        let entity = if let Some(entity) = self.free_entities.pop() {
            entity
        } else {
            let index = self.next_index;
            self.next_index += 1;
            Entity::new(index, 0)
        };
        self.bitset.set(entity.index(), true);
        if entity.index() as usize >= self.versions.len() {
            self.versions.resize(entity.index() as usize + 1, 0);
        }
        self.versions[entity.index() as usize] = entity.version();
        entity
    }

    pub(crate) fn destroy(&mut self, entity: Entity) {
        self.bitset.set(entity.index(), false);
        self.free_entities.push(Entity::new(
            entity.index(),
            entity.version().wrapping_add(1),
        ));
    }

    pub(crate) fn iter(&self) -> impl Iterator<Item = Entity> + '_ {
        self.bitset
            .iter()
            .map(move |index| Entity::new(index, self.versions[index as usize]))
    }
}
