use alloc::vec::Vec;
use mini3d_derive::Serialize;

use crate::container::ContainerTable;

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

#[derive(Default)]
pub(crate) struct EntityTable {
    free_entities: Vec<Entity>,
    next_index: EntityIndex, // Default index is 0
    pub(crate) bootstrap_stage: Entity,
    pub(crate) tick_stage: Entity,
    pub(crate) system_type: Entity,
    pub(crate) system_stage_type: Entity,
    pub(crate) identifier_type: Entity,
}

impl EntityTable {
    pub(crate) fn spawn(&mut self) -> Entity {
        if let Some(entity) = self.free_entities.pop() {
            entity
        } else {
            let index = self.next_index;
            self.next_index += 1;
            Entity::new(index, 0)
        }
    }

    pub(crate) fn despawn(&mut self, entity: Entity, containers: &mut ContainerTable) {
        // TODO: use sorted free entities to improve performance
        if self.free_entities.iter().all(|e| *e != entity) {
            self.free_entities.push(entity);
            // TODO: remove entity from containers
            // TODO: handle batch despawn
        }
    }
}
