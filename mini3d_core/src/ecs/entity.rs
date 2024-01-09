use core::cell::UnsafeCell;

use alloc::vec::Vec;

use crate::serialize::{Decoder, DecoderError, Encoder, EncoderError, Serialize};

use super::{
    archetype::{ArchetypeKey, ArchetypeTable},
    component::ComponentKey,
    container::ContainerTable,
    context::Context,
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

    pub fn raw(&self) -> u32 {
        self.0
    }

    // pub fn resolve(&mut self, resolver: &EntityResolver) {
    //     if let Some(handle) = resolver.map.get(&self.0) {
    //         self.0 = *handle;
    //     }
    // }

    /// Immediatlly effective
    pub fn create(ctx: &mut Context) -> Entity {
        let entity = ctx.ecs.entities.generate_entity();
        ctx.ecs.entity_created.push(entity);
        entity
    }

    /// Effective only at the end of system
    pub fn destroy(ctx: &mut Context, entity: Entity) {
        ctx.ecs.entity_destroyed.push(entity);
    }
}

impl Default for Entity {
    fn default() -> Self {
        Self::null()
    }
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
    pub(crate) archetype: ArchetypeKey,
    pub(crate) pool_index: u32,
}

pub(crate) struct EntityTable {
    pub(crate) archetypes: UnsafeCell<ArchetypeTable>,
    pub(crate) entries: PagedVector<EntityEntry>, // EntityKey -> EntityInfo
    pub(crate) free_entities: Vec<Entity>,
    pub(crate) next_entity: Entity,
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

    pub(crate) fn remove(
        &mut self,
        ctx: &mut Context,
        entity: Entity,
        containers: &mut ContainerTable,
    ) {
        let info = self.entries.get_mut(entity.key()).unwrap();
        // We can safely destroy the entity
        self.free_entities
            .push(Entity::new(entity.key(), entity.version() + 1));
        // Remove components from containers
        self.archetypes
            .get_mut()
            .components(info.archetype)
            .iter()
            .for_each(|component| {
                containers.remove_component(ctx, entity, *component);
            });
        // Remove the entity from the pool
        let archetype = &mut self.archetypes.get_mut()[info.archetype];
        let last_entity = archetype.pool.last().copied();
        archetype.pool.swap_remove(info.pool_index as usize);
        if let Some(last_entity) = last_entity {
            // Remap last entity
            self.entries.get_mut(last_entity.key()).unwrap().pool_index = info.pool_index;
        }
    }

    pub(crate) fn move_added_entity(
        &mut self,
        queries: &mut QueryTable,
        entity: Entity,
        component: ComponentKey,
    ) {
        let archetype = self.entries.get(entity.key()).unwrap().archetype;
        let new_archetype = self
            .archetypes
            .get_mut()
            .find_add(queries, archetype, component);
        self.move_entity(entity, new_archetype);
    }

    pub(crate) fn move_removed_entity(
        &mut self,
        queries: &mut QueryTable,
        entity: Entity,
        component: ComponentKey,
    ) {
        let archetype = self.entries.get(entity.key()).unwrap().archetype;
        let new_archetype = self
            .archetypes
            .get_mut()
            .find_remove(queries, archetype, component);
        self.move_entity(entity, new_archetype);
    }

    fn move_entity(&mut self, entity: Entity, new_archetype: ArchetypeKey) {
        // Find currrent archetype
        let entity_entry = self.entries.get(entity.key()).unwrap();
        let current_archetype = entity_entry.archetype;
        // Remove from current archetype
        let archetype = &mut self.archetypes.get_mut().entries[current_archetype];
        let last_entity = archetype.pool.last().copied();
        archetype.pool.swap_remove(entity_entry.pool_index as usize);
        if let Some(last_entity) = last_entity {
            // Remap last entity
            self.entries.get_mut(last_entity.key()).unwrap().pool_index = entity_entry.pool_index;
        }
        // Update archetype
        self.entries.get_mut(entity.key()).unwrap().archetype = new_archetype;
        // Add to new archetype
        self.archetypes.get_mut().entries[new_archetype]
            .pool
            .push(entity);
    }
}

impl Default for EntityTable {
    fn default() -> Self {
        Self {
            archetypes: UnsafeCell::new(ArchetypeTable::new()),
            entries: PagedVector::new(),
            free_entities: Vec::new(),
            next_entity: Entity::new(1, 0),
        }
    }
}
