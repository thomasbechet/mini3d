use std::ops::Range;

use crate::{
    registry::{
        component::{ComponentId, ComponentRegistry},
        error::RegistryError,
        system::System,
    },
    utils::{
        slotmap::{SlotId, SlotMap},
        uid::UID,
    },
};

use super::{
    archetype::{ArchetypeId, ArchetypeTable},
    entity::{Entity, EntityTable},
};

#[derive(Default, PartialEq, Eq, Clone, Copy)]
pub struct Query(SlotId);

#[derive(Default, PartialEq, Eq, Clone, Copy)]
pub struct FilterQuery(SlotId);

#[derive(PartialEq, Eq, Clone, Copy)]
pub(crate) enum FilterKind {
    Added,
    Removed,
    Changed,
}

#[derive(Default)]
pub(crate) struct QueryEntry {
    all: Range<usize>,
    any: Range<usize>,
    not: Range<usize>,
    archetypes: Vec<ArchetypeId>,
}

pub(crate) struct FilterQueryEntry {
    query: Query,
    cycle: usize,
    kind: FilterKind,
    entities: Vec<Entity>,
    system: System,
}

#[derive(Default)]
pub(crate) struct QueryTable {
    group_filters: Vec<ComponentId>,
    queries: SlotMap<QueryEntry>,
    pub(crate) filter_queries: SlotMap<FilterQueryEntry>,
}

impl QueryTable {
    fn query_match(
        &self,
        query: Query,
        archetype: ArchetypeId,
        archetypes: &ArchetypeTable,
    ) -> bool {
        let components = archetypes.components(archetype);
        let query = self.queries.get(query.0).unwrap();
        let all = &self.group_filters[query.all.clone()];
        let any = &self.group_filters[query.any.clone()];
        let not = &self.group_filters[query.not.clone()];
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

    fn find_same_query(
        &self,
        all: &[ComponentId],
        any: &[ComponentId],
        not: &[ComponentId],
    ) -> Option<Query> {
        for (id, query) in self.queries.iter() {
            if query.all.len() != all.len() {
                continue;
            }
            if query.any.len() != any.len() {
                continue;
            }
            if query.not.len() != not.len() {
                continue;
            }
            let all2 = &self.group_filters[query.all.clone()];
            let any2 = &self.group_filters[query.any.clone()];
            let not2 = &self.group_filters[query.not.clone()];
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
        archetypes: &mut ArchetypeTable,
        all: &[ComponentId],
        any: &[ComponentId],
        not: &[ComponentId],
    ) -> Query {
        let mut query = QueryEntry::default();
        let start = self.group_filters.len();
        self.group_filters.extend_from_slice(all);
        self.group_filters.extend_from_slice(any);
        self.group_filters.extend_from_slice(not);
        query.all = start..start + all.len();
        query.any = start + all.len()..start + all.len() + any.len();
        query.not = start + all.len() + any.len()..start + all.len() + any.len() + not.len();
        let id = Query(self.queries.add(query));
        for archetype in archetypes.iter() {
            if self.query_match(id, archetype, archetypes) {
                self.queries[id.0].archetypes.push(archetype);
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
            entities: Vec::new(),
            system,
        }));
        // Register to groups for added / removed events
        self.query_archetypes(query)
            .iter()
            .for_each(|archetype| entities.register_filter_query(*archetype, id, kind));
        id
    }

    pub(crate) fn query_archetypes(&self, id: Query) -> &[ArchetypeId] {
        &self.queries.get(id.0).unwrap().archetypes
    }

    pub(crate) fn filter_query(&self, id: FilterQuery) -> &[Entity] {
        &self.filter_queries.get(id.0).unwrap().entities
    }
}

pub struct QueryBuilder<'a> {
    pub(crate) registry: &'a ComponentRegistry,
    pub(crate) system: System,
    pub(crate) all: &'a mut Vec<ComponentId>,
    pub(crate) any: &'a mut Vec<ComponentId>,
    pub(crate) not: &'a mut Vec<ComponentId>,
    pub(crate) entities: &'a mut EntityTable,
    pub(crate) archetypes: &'a mut ArchetypeTable,
    pub(crate) queries: &'a mut QueryTable,
}

impl<'a> QueryBuilder<'a> {
    pub fn all(self, components: &[UID]) -> Result<Self, RegistryError> {
        for component in components {
            let component = self
                .registry
                .find_id(*component)
                .ok_or(RegistryError::ComponentNotFound)?;
            if self.all.iter().all(|c| *c != component) {
                self.all.push(component);
            }
        }
        Ok(self)
    }

    pub fn any(self, components: &[UID]) -> Result<Self, RegistryError> {
        for component in components {
            let component = self
                .registry
                .find_id(*component)
                .ok_or(RegistryError::ComponentNotFound)?;
            if self.any.iter().all(|c| *c != component) {
                self.any.push(component);
            }
        }
        Ok(self)
    }

    pub fn not(self, components: &[UID]) -> Result<Self, RegistryError> {
        for component in components {
            let component = self
                .registry
                .find_id(*component)
                .ok_or(RegistryError::ComponentNotFound)?;
            if self.not.iter().all(|c| *c != component) {
                self.not.push(component);
            }
        }
        Ok(self)
    }

    pub fn build(self) -> Query {
        if let Some(id) = self.queries.find_same_query(self.all, self.any, self.not) {
            return id;
        }
        self.queries
            .add_query(self.entities, self.archetypes, self.all, self.any, self.not)
    }

    fn add_filter_query(self, kind: FilterKind) -> FilterQuery {
        let id = self
            .queries
            .find_same_query(self.all, self.any, self.not)
            .unwrap_or_else(|| {
                self.queries
                    .add_query(self.entities, self.archetypes, self.all, self.any, self.not)
            });
        self.queries
            .add_filter_query(self.entities, kind, id, self.system)
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
