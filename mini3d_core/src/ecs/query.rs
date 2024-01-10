use core::{cell::UnsafeCell, ops::Range};

use alloc::{boxed::Box, vec::Vec};

use crate::{
    slot_map_key,
    utils::{
        slotmap::SlotMap,
        uid::{ToUID, UID},
    },
};

use super::{
    archetype::{ArchetypeEntry, ArchetypeKey, ArchetypeTable},
    component::ComponentKey,
    container::ContainerTable,
    entity::{Entity, EntityTable},
    error::ResolverError,
    system::Resolver,
};

#[derive(PartialEq, Eq, Clone, Copy)]
pub struct Query {
    pub(crate) query: *const QueryEntry,
    pub(crate) archetypes: *const ArchetypeTable,
}

impl Default for Query {
    fn default() -> Self {
        Self {
            query: core::ptr::null(),
            archetypes: core::ptr::null(),
        }
    }
}

impl Query {
    pub fn resolve<'a>(&'a mut self, resolver: &'a mut Resolver) -> QueryBuilder<'a> {
        self.archetypes = resolver.entities.archetypes.get();
        resolver.all.clear();
        resolver.any.clear();
        resolver.not.clear();
        QueryBuilder {
            query: &mut self.query,
            all: resolver.all,
            any: resolver.any,
            not: resolver.not,
            entities: resolver.entities,
            queries: resolver.queries,
            containers: resolver.containers,
        }
    }

    pub fn iter(&'_ self) -> impl Iterator<Item = Entity> + '_ {
        unsafe { &*self.query }
            .archetypes
            .iter()
            .flat_map(|archetype| {
                unsafe { &*self.archetypes }
                    .entries
                    .get(*archetype)
                    .unwrap()
                    .pool
                    .iter()
                    .copied()
            })
    }
}

#[derive(Default)]
pub(crate) struct QueryEntry {
    pub(crate) all: Range<usize>,
    pub(crate) any: Range<usize>,
    pub(crate) not: Range<usize>,
    pub(crate) archetypes: Vec<ArchetypeKey>,
}

slot_map_key!(QueryKey);

#[derive(Default)]
pub(crate) struct QueryTable {
    pub(crate) components: Vec<ComponentKey>,
    pub(crate) entries: SlotMap<QueryKey, Box<UnsafeCell<QueryEntry>>>,
}

pub(crate) fn query_archetype_match(
    query: &QueryEntry,
    query_components: &[ComponentKey],
    archetype: &ArchetypeEntry,
    archetype_components: &[ComponentKey],
) -> bool {
    let components = &archetype_components[archetype.component_range.clone()];
    let all = &query_components[query.all.clone()];
    let any = &query_components[query.any.clone()];
    let not = &query_components[query.not.clone()];
    // All check
    if !all.is_empty() {
        for c in all {
            if !components.contains(c) {
                return false;
            }
        }
    }
    // Any check
    if !any.is_empty() {
        let mut found = false;
        for c in any {
            if components.contains(c) {
                found = true;
                break;
            }
        }
        if !found {
            return false;
        }
    }
    // Not check
    if !not.is_empty() {
        for c in not {
            if components.contains(c) {
                return false;
            }
        }
    }
    true
}

impl QueryTable {
    fn find_same_query(
        &self,
        all: &[ComponentKey],
        any: &[ComponentKey],
        not: &[ComponentKey],
    ) -> Option<QueryKey> {
        for (id, query) in self.entries.iter() {
            let query = unsafe { &*query.get() };
            if query.all.len() != all.len() {
                continue;
            }
            if query.any.len() != any.len() {
                continue;
            }
            if query.not.len() != not.len() {
                continue;
            }
            let all2 = &self.components[query.all.clone()];
            let any2 = &self.components[query.any.clone()];
            let not2 = &self.components[query.not.clone()];
            if all.iter().any(|c| !all2.contains(c)) {
                continue;
            }
            if any.iter().any(|c| !any2.contains(c)) {
                continue;
            }
            if not.iter().any(|c| !not2.contains(c)) {
                continue;
            }
            return Some(id);
        }
        None
    }

    fn add_query(
        &mut self,
        entities: &mut EntityTable,
        all: &[ComponentKey],
        any: &[ComponentKey],
        not: &[ComponentKey],
    ) -> QueryKey {
        let mut query = QueryEntry::default();
        let start = self.components.len();
        self.components.extend_from_slice(all);
        self.components.extend_from_slice(any);
        self.components.extend_from_slice(not);
        query.all = start..start + all.len();
        query.any = start + all.len()..start + all.len() + any.len();
        query.not = start + all.len() + any.len()..start + all.len() + any.len() + not.len();
        let id = self.entries.add(Box::new(UnsafeCell::new(query)));
        // Bind new query to existing archetypes
        let archetypes = entities.archetypes.get_mut();
        for archetype in archetypes.entries.keys() {
            let archetype_entry = &archetypes[archetype];
            let query_entry = self.entries[id].get_mut();
            if query_archetype_match(
                query_entry,
                &self.components,
                archetype_entry,
                &archetypes.components,
            ) {
                self.entries[id].get_mut().archetypes.push(archetype);
            }
        }
        id
    }
}

pub struct QueryBuilder<'a> {
    pub(crate) query: &'a mut *const QueryEntry,
    pub(crate) all: &'a mut Vec<ComponentKey>,
    pub(crate) any: &'a mut Vec<ComponentKey>,
    pub(crate) not: &'a mut Vec<ComponentKey>,
    pub(crate) entities: &'a mut EntityTable,
    pub(crate) containers: &'a mut ContainerTable,
    pub(crate) queries: &'a mut QueryTable,
}

impl<'a> QueryBuilder<'a> {
    fn find_component(&mut self, component: UID) -> Result<ComponentKey, ResolverError> {
        self.containers
            .find(component)
            .ok_or(ResolverError::ComponentNotFound)
    }

    pub fn all(mut self, components: &[impl ToUID]) -> Result<Self, ResolverError> {
        for component in components {
            let id = self.find_component(component.to_uid())?;
            if self.all.iter().all(|c| *c != id) {
                self.all.push(id);
            }
        }
        Ok(self)
    }

    pub fn any(mut self, components: &[impl ToUID]) -> Result<Self, ResolverError> {
        for component in components {
            let id = self.find_component(component.to_uid())?;
            if self.any.iter().all(|c| *c != id) {
                self.any.push(id);
            }
        }
        Ok(self)
    }

    pub fn not(mut self, components: &[impl ToUID]) -> Result<Self, ResolverError> {
        for component in components {
            let id = self.find_component(component.to_uid())?;
            if self.not.iter().all(|c| *c != id) {
                self.not.push(id);
            }
        }
        Ok(self)
    }
}

impl<'a> Drop for QueryBuilder<'a> {
    fn drop(&mut self) {
        if let Some(id) = self.queries.find_same_query(self.all, self.any, self.not) {
            *self.query = self.queries.entries.get(id).unwrap().get();
        }
        let id = self
            .queries
            .add_query(self.entities, self.all, self.any, self.not);
        *self.query = self.queries.entries.get(id).unwrap().get();
    }
}