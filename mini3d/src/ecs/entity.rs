use crate::serialize::{Decoder, DecoderError, Encoder, EncoderError, Serialize};

use super::{
    archetype::{Archetype, ArchetypeTable},
    container::ContainerTable,
    sparse::PagedVector,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Entity(pub(crate) u32);

pub(crate) type EntityVersion = u8;
pub(crate) type EntityKey = u32;

impl Entity {
    pub(crate) fn new(key: EntityKey, version: EntityVersion) -> Self {
        Self(key | ((version as EntityKey) << 24))
    }

    pub(crate) fn key(&self) -> EntityKey {
        self.0 & 0x00ff_ffff
    }

    pub(crate) fn version(&self) -> EntityVersion {
        (self.0 >> 24) as EntityVersion
    }

    pub fn null() -> Self {
        Self(0)
    }

    // pub fn resolve(&mut self, resolver: &EntityResolver) {
    //     if let Some(handle) = resolver.map.get(&self.0) {
    //         self.0 = *handle;
    //     }
    // }
}

impl Serialize for Entity {
    type Header = ();

    fn serialize(&self, encoder: &mut impl Encoder) -> Result<(), EncoderError> {
        encoder.write_u32(self.0)?;
        Ok(())
    }

    fn deserialize(
        decoder: &mut impl Decoder,
        _header: &Self::Header,
    ) -> Result<Self, DecoderError> {
        Ok(Self(decoder.read_u32()?))
    }
}

pub(crate) enum EntityChange {
    Added(Entity),
    Removed(Entity),
}

#[derive(Default, Clone, Copy)]
pub(crate) struct EntityEntry {
    pub(crate) archetype: Archetype,
    pub(crate) pool_index: u32,
}

pub(crate) struct EntityTable {
    pub(crate) archetypes: ArchetypeTable,
    pub(crate) entries: PagedVector<EntityEntry>, // EntityKey -> EntityInfo
    pub(crate) free_entities: Vec<Entity>,
    pub(crate) next_entity: Entity,
    pub(crate) changes: Vec<EntityChange>,
}

impl EntityTable {
    pub(crate) fn generate_entity(&mut self) -> Entity {
        if let Some(entity) = self.free_entities.pop() {
            return entity;
        }
        let entity = self.next_entity;
        self.next_entity = Entity::new(entity.key() + 1, 0);
        entity
    }

    pub(crate) fn remove(&mut self, entity: Entity, containers: &mut ContainerTable) {
        let info = self.entries.get_mut(entity.key()).unwrap();
        // We can safely destroy the entity
        self.free_entities
            .push(Entity::new(entity.key(), entity.version() + 1));
        // Remove components from containers
        self.archetypes
            .components(info.archetype)
            .iter()
            .for_each(|component| {
                containers.remove(entity, *component);
            });
        // In all cases, we want to remove the entity from the pool
        let archetype = &mut self.archetypes[info.archetype];
        let last_entity = archetype.pool.last().copied();
        archetype.pool.swap_remove(info.pool_index as usize);
        if let Some(last_entity) = last_entity {
            // Remap last entity
            self.entries.get_mut(last_entity.key()).unwrap().pool_index = info.pool_index;
        }
    }

    pub(crate) fn iter_pool_entities(
        &self,
        archetype: Archetype,
    ) -> impl Iterator<Item = Entity> + '_ {
        if let Some(archetype) = self.archetypes.entries.get(archetype) {
            archetype.pool.iter().copied()
        } else {
            [].iter().copied()
        }
    }
}

impl Default for EntityTable {
    fn default() -> Self {
        Self {
            archetypes: ArchetypeTable::new(),
            entries: PagedVector::new(),
            free_entities: Vec::new(),
            next_entity: Entity::new(1, 0),
            changes: Vec::new(),
        }
    }
}
