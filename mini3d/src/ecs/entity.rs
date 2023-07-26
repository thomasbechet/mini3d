use crate::{
    serialize::{Decoder, DecoderError, Encoder, EncoderError, Serialize},
    utils::slotmap::SecondaryMap,
};

use super::{
    archetype::{ArchetypeId, ArchetypeTable},
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

#[derive(Default, Clone, Copy)]
pub(crate) struct EntityInfo {
    pub(crate) archetype: ArchetypeId,
    pub(crate) pool_index: usize,
}

pub(crate) struct EntityTable {
    entities: PagedVector<EntityInfo>,
    free_entities: Vec<Entity>,
    next_entity: Entity,
    pools: SecondaryMap<Vec<Entity>>,
}

impl EntityTable {
    pub(crate) fn add(&mut self, archetypes: &ArchetypeTable) -> Entity {
        // Find next entitiy
        if let Some(entity) = self.free_entities.pop() {
            return entity;
        }
        let entity = self.next_entity;
        self.next_entity = Entity::new(entity.key() + 1, 0);
        // Put the entity in the empty pool
        self.add_to_pool(entity, archetypes.empty);
        entity
    }

    fn get_pool(&mut self, archetype: ArchetypeId) -> &mut Vec<Entity> {
        if let Some(pool) = self.pools.get_mut(archetype) {
            return pool;
        } else {
            self.pools.insert(archetype, Vec::default());
            return &mut self.pools[archetype];
        }
    }

    fn add_to_pool(&mut self, entity: Entity, archetype: ArchetypeId) {
        let pool = self.get_pool(archetype);
        let pool_index = pool.len();
        pool.push(entity);
        self.entities.set(
            entity.key(),
            EntityInfo {
                archetype,
                pool_index,
            },
        );
    }

    fn remove_from_pool(&mut self, entity: Entity) {
        // Swap remove entity from pool
        let info = *self.entities.get(entity.key()).unwrap();
        let pool = self.get_pool(info.archetype);
        let last_entity = pool.last().copied();
        pool.swap_remove(info.pool_index);
        if let Some(last_entity) = last_entity {
            self.entities.set(last_entity.key(), info);
        }
    }

    pub(crate) fn remove(&mut self, entity: Entity) {
        // Remove from pool
        self.remove_from_pool(entity);
        // Add entity to free list
        self.free_entities
            .push(Entity::new(entity.key(), entity.version() + 1));
    }

    pub(crate) fn set_entity_archetype(&mut self, entity: Entity, archetype: ArchetypeId) {
        // Remove from current pool
        self.remove_from_pool(entity);
        // Add to new pool
        self.add_to_pool(entity, archetype);
    }
}

impl Default for EntityTable {
    fn default() -> Self {
        Self {
            entities: PagedVector::new(),
            free_entities: Vec::new(),
            next_entity: Entity::new(1, 0),
            pools: SecondaryMap::default(),
        }
    }
}
