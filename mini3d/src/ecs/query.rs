use std::ops::Range;

use crate::{
    registry::{
        component::{ComponentId, ComponentRegistry},
        error::RegistryError,
        system::System,
    },
    utils::{
        slotmap::{SlotId, SlotMap},
        uid::ToUID,
    },
};

use super::{
    archetype::{Archetype, ArchetypeEntry},
    entity::{Entity, EntityTable},
};

#[derive(Default, PartialEq, Eq, Clone, Copy)]
pub struct Query(pub(crate) SlotId);

#[derive(Default, Debug, PartialEq, Eq, Clone, Copy)]
pub struct FilterQuery(pub(crate) SlotId);

#[derive(PartialEq, Eq, Clone, Copy)]
pub(crate) enum FilterKind {
    Added,
    Removed,
    Changed,
}

#[derive(Default)]
pub(crate) struct QueryEntry {
    pub(crate) all: Range<usize>,
    pub(crate) any: Range<usize>,
    pub(crate) not: Range<usize>,
    pub(crate) archetypes: Vec<Archetype>,
}

pub(crate) struct FilterQueryEntry {
    pub(crate) query: Query,
    pub(crate) cycle: usize,
    pub(crate) kind: FilterKind,
    pub(crate) pool: Vec<Entity>,
    system: System,
}

#[derive(Default)]
pub(crate) struct QueryTable {
    pub(crate) components: Vec<ComponentId>,
    pub(crate) entries: SlotMap<QueryEntry>,
    pub(crate) filter_queries: SlotMap<FilterQueryEntry>,
}

pub(crate) fn query_archetype_match(
    query: &QueryEntry,
    query_components: &[ComponentId],
    archetype: &ArchetypeEntry,
    archetype_components: &[ComponentId],
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
        all: &[ComponentId],
        any: &[ComponentId],
        not: &[ComponentId],
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
        all: &[ComponentId],
        any: &[ComponentId],
        not: &[ComponentId],
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

    fn add_filter_query(
        &mut self,
        entities: &mut EntityTable,
        kind: FilterKind,
        query: Query,
        system: System,
    ) -> FilterQuery {
        let id = FilterQuery(self.filter_queries.add(FilterQueryEntry {
            query,
            cycle: 0,
            kind,
            pool: Vec::new(),
            system,
        }));
        // Bind existing archetypes to new filter
        match kind {
            FilterKind::Added => {
                for archetype in self.entries[query.0].archetypes.iter() {
                    entities.archetypes.entries[*archetype]
                        .added_filter_queries
                        .push(id);
                }
            }
            FilterKind::Removed => {
                for archetype in self.entries[query.0].archetypes.iter() {
                    entities.archetypes.entries[*archetype]
                        .removed_filter_queries
                        .push(id);
                }
            }
            FilterKind::Changed => todo!(),
        }
        id
    }
}

pub struct QueryBuilder<'a> {
    pub(crate) registry: &'a ComponentRegistry,
    pub(crate) system: System,
    pub(crate) all: &'a mut Vec<ComponentId>,
    pub(crate) any: &'a mut Vec<ComponentId>,
    pub(crate) not: &'a mut Vec<ComponentId>,
    pub(crate) entities: &'a mut EntityTable,
    pub(crate) queries: &'a mut QueryTable,
    pub(crate) filter_queries: &'a mut Vec<FilterQuery>,
}

impl<'a> QueryBuilder<'a> {
    pub fn all(self, components: &[impl ToUID]) -> Result<Self, RegistryError> {
        for component in components {
            let component = self
                .registry
                .find_id(component.to_uid())
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
                .find_id(component.to_uid())
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
                .find_id(component.to_uid())
                .ok_or(RegistryError::ComponentNotFound)?;
            if self.not.iter().all(|c| *c != component) {
                self.not.push(component);
            }
        }
        Ok(self)
    }

    fn build_query(&mut self) -> Query {
        if let Some(id) = self.queries.find_same_query(self.all, self.any, self.not) {
            return id;
        }
        self.queries
            .add_query(self.entities, self.all, self.any, self.not)
    }

    pub fn build(mut self) -> Query {
        self.build_query()
    }

    fn add_filter_query(mut self, kind: FilterKind) -> FilterQuery {
        // Build base query
        let query = self.build_query();
        // Add filtered query
        let id = self
            .queries
            .add_filter_query(self.entities, kind, query, self.system);
        // Keep reference of filter in instance
        self.filter_queries.push(id);
        id
    }

    pub fn added(self) -> FilterQuery {
        self.add_filter_query(FilterKind::Added)
    }

    pub fn removed(self) -> FilterQuery {
        self.add_filter_query(FilterKind::Removed)
    }

    pub fn changed(self, component: ComponentId) -> FilterQuery {
        self.add_filter_query(FilterKind::Changed)
    }
}
