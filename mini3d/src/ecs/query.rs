use std::ops::Range;

use crate::{
    feature::common::system::System,
    utils::{
        slotmap::{SlotId, SlotMap},
        uid::ToUID,
    },
};

use super::{
    archetype::{Archetype, ArchetypeEntry},
    entity::EntityTable,
};

#[derive(Default, PartialEq, Eq, Clone, Copy)]
pub struct Query(pub(crate) SlotId);

#[derive(Default)]
pub(crate) struct QueryEntry {
    pub(crate) all: Range<usize>,
    pub(crate) any: Range<usize>,
    pub(crate) not: Range<usize>,
    pub(crate) archetypes: Vec<Archetype>,
}

#[derive(Default)]
pub(crate) struct QueryTable {
    pub(crate) components: Vec<ComponentType>,
    pub(crate) entries: SlotMap<QueryEntry>,
}

pub(crate) fn query_archetype_match(
    query: &QueryEntry,
    query_components: &[ComponentType],
    archetype: &ArchetypeEntry,
    archetype_components: &[ComponentType],
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
        all: &[ComponentType],
        any: &[ComponentType],
        not: &[ComponentType],
    ) -> Option<Query> {
        for (id, query) in self.entries.iter() {
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
            return Some(Query(id));
        }
        None
    }

    fn add_query(
        &mut self,
        entities: &mut EntityTable,
        all: &[ComponentType],
        any: &[ComponentType],
        not: &[ComponentType],
    ) -> Query {
        let mut query = QueryEntry::default();
        let start = self.components.len();
        self.components.extend_from_slice(all);
        self.components.extend_from_slice(any);
        self.components.extend_from_slice(not);
        query.all = start..start + all.len();
        query.any = start + all.len()..start + all.len() + any.len();
        query.not = start + all.len() + any.len()..start + all.len() + any.len() + not.len();
        let id = Query(self.entries.add(query));
        // Bind new query to existing archetypes
        for archetype in entities.archetypes.entries.keys() {
            let archetype_entry = &entities.archetypes[archetype];
            let query_entry = &self.entries[id.0];
            if query_archetype_match(
                query_entry,
                &self.components,
                archetype_entry,
                &entities.archetypes.components,
            ) {
                self.entries[id.0].archetypes.push(archetype);
            }
        }
        id
    }
}

pub struct QueryBuilder<'a> {
    pub(crate) registry: &'a ComponentRegistryManager,
    pub(crate) system: System,
    pub(crate) all: &'a mut Vec<ComponentType>,
    pub(crate) any: &'a mut Vec<ComponentType>,
    pub(crate) not: &'a mut Vec<ComponentType>,
    pub(crate) entities: &'a mut EntityTable,
    pub(crate) queries: &'a mut QueryTable,
}

impl<'a> QueryBuilder<'a> {
    pub fn all(self, components: &[impl ToUID]) -> Result<Self, RegistryError> {
        for component in components {
            let component = self
                .registry
                .find(component.to_uid())
                .ok_or(RegistryError::ComponentNotFound)?;
            if self.all.iter().all(|c| *c != component) {
                self.all.push(component);
            }
        }
        Ok(self)
    }

    pub fn any(self, components: &[impl ToUID]) -> Result<Self, RegistryError> {
        for component in components {
            let component = self
                .registry
                .find(component.to_uid())
                .ok_or(RegistryError::ComponentNotFound)?;
            if self.any.iter().all(|c| *c != component) {
                self.any.push(component);
            }
        }
        Ok(self)
    }

    pub fn not(self, components: &[impl ToUID]) -> Result<Self, RegistryError> {
        for component in components {
            let component = self
                .registry
                .find(component.to_uid())
                .ok_or(RegistryError::ComponentNotFound)?;
            if self.not.iter().all(|c| *c != component) {
                self.not.push(component);
            }
        }
        Ok(self)
    }

    pub fn build(mut self) -> Query {
        if let Some(id) = self.queries.find_same_query(self.all, self.any, self.not) {
            return id;
        }
        self.queries
            .add_query(self.entities, self.all, self.any, self.not)
    }
}
