use crate::{
    serialize::{Decoder, DecoderError, Encoder, EncoderError, Serialize},
    utils::slotmap::SecondaryMap,
};

use super::{
    archetype::{ArchetypeId, ArchetypeTable},
    query::QueryId,
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
    pub(crate) group_index: u32,
}

#[derive(Default)]
pub(crate) struct EntityGroup {
    entities: Vec<Entity>,
    query: Vec<QueryId>,
}

pub(crate) struct EntityTable {
    entities: PagedVector<EntityInfo>, // EntityKey -> EntityInfo
    free_entities: Vec<Entity>,
    next_entity: Entity,
    groups: SecondaryMap<EntityGroup>, // ArchetypeId -> EntityGroup
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
        self.add_to_group(entity, archetypes.empty);
        entity
    }

    fn group(&mut self, archetype: ArchetypeId) -> &mut EntityGroup {
        if let Some(group) = self.groups.get_mut(archetype) {
            return group;
        } else {
            self.groups.insert(archetype, Vec::default());
            return &mut self.groups[archetype];
        }
    }

    fn add_to_group(&mut self, entity: Entity, archetype: ArchetypeId) {
        let group = self.group(archetype);
        let group_index = group.len();
        group.push(entity);
        self.entities.set(
            entity.key(),
            EntityInfo {
                archetype,
                group_index,
            },
        );
    }

    fn remove_from_group(&mut self, entity: Entity) {
        // Swap remove entity from group
        let info = *self.entities.get(entity.key()).unwrap();
        let group = self.group(info.archetype);
        let last_entity = group.last().copied();
        group.swap_remove(info.group_index);
        if let Some(last_entity) = last_entity {
            self.entities.set(last_entity.key(), info);
        }
    }

    pub(crate) fn remove(&mut self, entity: Entity) {
        // Remove from group
        self.remove_from_group(entity);
        // Add entity to free list
        self.free_entities
            .push(Entity::new(entity.key(), entity.version() + 1));
    }

    pub(crate) fn change_archetype(&mut self, entity: Entity, archetype: ArchetypeId) {
        // Remove from current group
        self.remove_from_group(entity);
        // Add to new group
        self.add_to_group(entity, archetype);
    }

    pub(crate) fn get_archetype(&self, entitiy: Entity) -> ArchetypeId {
        self.entities.get(entitiy.key()).unwrap().archetype
    }
}

impl Default for EntityTable {
    fn default() -> Self {
        Self {
            entities: PagedVector::new(),
            free_entities: Vec::new(),
            next_entity: Entity::new(1, 0),
            groups: SecondaryMap::default(),
        }
    }
}
