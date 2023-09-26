use crate::{
    registry::component::{ComponentHandle, PrivateComponentTableMut},
    serialize::{Decoder, DecoderError, Encoder, EncoderError, Serialize},
};

use super::{
    archetype::{Archetype, ArchetypeTable},
    container::ContainerTable,
    query::QueryTable,
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
pub(crate) struct EntityEntry {
    pub(crate) archetype: Archetype,
    pub(crate) pool_index: u32,
    pub(crate) pending_remove_counter: u16,
}

pub(crate) struct EntityTable {
    pub(crate) archetypes: ArchetypeTable,
    pub(crate) entries: PagedVector<EntityEntry>, // EntityKey -> EntityInfo
    free_entities: Vec<Entity>,
    next_entity: Entity,
}

impl EntityTable {
    fn next_entity(&mut self) -> Entity {
        if let Some(entity) = self.free_entities.pop() {
            return entity;
        }
        let entity = self.next_entity;
        self.next_entity = Entity::new(entity.key() + 1, 0);
        entity
    }

    pub(crate) fn remove(&mut self, entity: Entity, containers: &mut ContainerTable) {
        let info = self.entries.get_mut(entity.key()).unwrap();
        // Check if entity is already pending removal
        if info.pending_remove_counter > 0 {
            return;
        }
        // Check if the archetype is watched by any filter queries
        let archetype = &self.archetypes.entries[info.archetype];
        if !archetype.removed_filter_queries.is_empty() {
            // Entity must be removed manually by the filter query
            info.pending_remove_counter = archetype.removed_filter_queries.len() as u16;
        } else {
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
        }
        // In all cases, we want to remove the entity from the pool
        // Filtered queries are not affected as they keep their own list of entities
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
        }
    }
}

pub struct EntityBuilder<'a> {
    entity: Entity,
    archetype: Archetype,
    entities: &'a mut EntityTable,
    containers: &'a mut ContainerTable,
    queries: &'a mut QueryTable,
    cycle: u32,
}

impl<'a> EntityBuilder<'a> {
    pub(crate) fn new(
        entities: &'a mut EntityTable,
        containers: &'a mut ContainerTable,
        queries: &'a mut QueryTable,
        cycle: u32,
    ) -> Self {
        // Find next entity
        let entity = entities.next_entity();
        Self {
            entity,
            archetype: entities.archetypes.empty,
            entities,
            containers,
            queries,
            cycle,
        }
    }

    pub fn with<H: ComponentHandle>(mut self, component: H, data: H::Data) -> Self {
        self.archetype =
            self.entities
                .archetypes
                .find_add(self.queries, self.archetype, component.id());
        component.insert_single_container(
            PrivateComponentTableMut(self.containers),
            self.entity,
            data,
            self.cycle,
        );
        self
    }

    pub fn with_array<H: ComponentHandle>(mut self, component: H, data: &[H::Data]) -> Self {
        self
    }

    pub fn with_list<H: ComponentHandle>(mut self, component: H, data: &[H::Data]) -> Self {
        self
    }

    pub fn with_map<H: ComponentHandle>(
        mut self,
        component: H,
        data: &[(Entity, H::Data)],
    ) -> Self {
        self
    }

    pub fn with_default<H: ComponentHandle>(self, component: H) -> Self {
        self.with(component, H::Data::default())
    }

    pub fn build(self) -> Entity {
        self.entity
    }
}

impl<'a> Drop for EntityBuilder<'a> {
    fn drop(&mut self) {
        // Add to pool
        let archetype = &mut self.entities.archetypes[self.archetype];
        let pool_index = archetype.pool.len();
        archetype.pool.push(self.entity);
        // Add to added filter queries
        for added in &archetype.added_filter_queries {
            self.queries.filter_queries[added.0].pool.push(self.entity);
        }
        // Update entity info
        self.entities.entries.set(
            self.entity.key(),
            EntityEntry {
                archetype: self.archetype,
                pool_index: pool_index as u32,
                pending_remove_counter: 0,
            },
        );
    }
}
