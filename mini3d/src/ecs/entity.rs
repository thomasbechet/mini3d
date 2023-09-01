use crate::{
    registry::component::{ComponentHandle, ComponentId, PrivateComponentTableMut},
    serialize::{Decoder, DecoderError, Encoder, EncoderError, Serialize},
    utils::slotmap::SecondaryMap,
};

use super::{
    archetype::{ArchetypeId, ArchetypeTable},
    component::ComponentTable,
    query::{FilterKind, FilterQuery},
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
    pub(crate) group: ArchetypeId,
    pub(crate) group_index: u32,
    pub(crate) pending_remove_counter: u16,
}

#[derive(Default)]
pub(crate) struct EntityGroup {
    entities: Vec<Entity>,
    added_filter_queries: Vec<FilterQuery>,
    removed_filter_queries: Vec<FilterQuery>,
}

pub(crate) struct EntityTable {
    entities: PagedVector<EntityInfo>, // EntityKey -> EntityInfo
    free_entities: Vec<Entity>,
    next_entity: Entity,
    groups: SecondaryMap<EntityGroup>, // ArchetypeId -> EntityGroup
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

    fn get_or_create_group(&mut self, group: ArchetypeId) -> &mut EntityGroup {
        if !self.groups.contains(group) {
            self.groups.insert(group, EntityGroup::default());
        }
        &mut self.groups[group]
    }

    fn add_to_group(&mut self, entity: Entity, archetype: ArchetypeId) {
        let group = self.get_or_create_group(archetype);
        let group_index = group.entities.len() as u32;
        group.entities.push(entity);
        self.entities.set(
            entity.key(),
            EntityInfo {
                group: archetype,
                group_index,
                pending_remove_counter: 0,
            },
        );
    }

    pub(crate) fn remove(
        &mut self,
        entity: Entity,
        archetypes: &mut ArchetypeTable,
        components: &mut ComponentTable,
    ) {
        let info = self.entities.get_mut(entity.key()).unwrap();
        let group = &mut self.groups[info.group];
        // Check if the group is watched by any filter queries
        if !group.removed_filter_queries.is_empty() {
            // Entity must be removed manually by the filter query
            info.pending_remove_counter = group.removed_filter_queries.len() as u16;
        } else {
            // We can safely reuse the entity
            self.free_entities
                .push(Entity::new(entity.key(), entity.version() + 1));
        }
        // Remove entity from group (not required for filter queries)
        let last_entity = group.entities.last().copied();
        group.entities.swap_remove(info.group_index as usize);
        if let Some(last_entity) = last_entity {
            // Remap last entity
            self.entities
                .get_mut(last_entity.key())
                .unwrap()
                .group_index = info.group_index;
        }
        // Check if the group is watched by any filter queries
        if group.removed_filter_queries.is_empty() {
            let archetype = self.archetype(entity);
            // Remove components
            archetypes
                .components(archetype)
                .iter()
                .for_each(|component| {
                    components.remove(entity, *component);
                });
        }
    }

    pub(crate) fn archetype(&self, entitiy: Entity) -> ArchetypeId {
        self.entities.get(entitiy.key()).unwrap().group
    }

    pub(crate) fn iter_group_entities(
        &self,
        archetype: ArchetypeId,
    ) -> impl Iterator<Item = Entity> + '_ {
        let group = self.groups.get(archetype).unwrap();
        group.entities.iter().copied()
    }

    pub(crate) fn register_filter_query(
        &mut self,
        group: ArchetypeId,
        query: FilterQuery,
        kind: FilterKind,
    ) {
        match kind {
            FilterKind::Added => {
                self.get_or_create_group(group)
                    .added_filter_queries
                    .push(query);
            }
            FilterKind::Removed => {
                self.get_or_create_group(group)
                    .removed_filter_queries
                    .push(query);
            }
            FilterKind::Changed => {}
        }
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

pub struct EntityBuilder<'a> {
    entity: Entity,
    archetype: ArchetypeId,
    archetypes: &'a mut ArchetypeTable,
    entities: &'a mut EntityTable,
    components: &'a mut ComponentTable,
    cycle: u32,
}

impl<'a> EntityBuilder<'a> {
    pub(crate) fn new(
        archetypes: &'a mut ArchetypeTable,
        entities: &'a mut EntityTable,
        components: &'a mut ComponentTable,
        cycle: u32,
    ) -> Self {
        // Find next entity
        let entity = entities.next_entity();
        Self {
            entity,
            archetype: archetypes.empty,
            archetypes,
            entities,
            components,
            cycle,
        }
    }

    fn update_archetype(&mut self, component: ComponentId) {
        self.archetype = self.archetypes.find_add(self.archetype, component);
    }

    pub fn with<H: ComponentHandle>(mut self, component: H, data: H::Data) -> Self {
        self.update_archetype(component.id());
        component.insert_container(
            PrivateComponentTableMut(self.components),
            self.entity,
            data,
            self.cycle,
        );
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
        self.entities.add_to_group(self.entity, self.archetype);
    }
}
