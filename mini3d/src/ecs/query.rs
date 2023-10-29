use std::ops::Range;

use mini3d_derive::Error;

use crate::{
    api::Context,
    feature::core::component::ComponentId,
    resource::{handle::ResourceHandle, ResourceManager},
    utils::{
        slotmap::{SlotId, SlotMap},
        uid::ToUID,
    },
};

use super::{
    archetype::{Archetype, ArchetypeEntry},
    container::ContainerTable,
    entity::{Entity, EntityTable},
    system::SystemInstanceId,
};

#[derive(Default, PartialEq, Eq, Clone, Copy)]
pub struct Query(pub(crate) SlotId);

impl Query {
    pub fn query(&self, ctx: &Context) -> impl Iterator<Item = Entity> {
        ctx.queries.entries[self.0]
            .archetypes
            .iter()
            .flat_map(|archetype| ctx.entities.iter_pool_entities(*archetype))
    }
}

#[derive(Default)]
pub(crate) struct QueryEntry {
    pub(crate) all: Range<usize>,
    pub(crate) any: Range<usize>,
    pub(crate) not: Range<usize>,
    pub(crate) archetypes: Vec<Archetype>,
}

#[derive(Default)]
pub(crate) struct QueryTable {
    pub(crate) components: Vec<ComponentId>,
    pub(crate) entries: SlotMap<QueryEntry>,
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
}

#[derive(Error)]
pub enum QueryError {
    #[error("component not found")]
    ComponentNotFound,
}

pub struct QueryBuilder<'a> {
    pub(crate) system: SystemInstanceId,
    pub(crate) component_type: ResourceHandle,
    pub(crate) all: &'a mut Vec<ComponentId>,
    pub(crate) any: &'a mut Vec<ComponentId>,
    pub(crate) not: &'a mut Vec<ComponentId>,
    pub(crate) entities: &'a mut EntityTable,
    pub(crate) containers: &'a mut ContainerTable,
    pub(crate) queries: &'a mut QueryTable,
    pub(crate) resources: &'a mut ResourceManager,
}

impl<'a> QueryBuilder<'a> {
    fn find_component(&self, component: impl ToUID) -> Result<ComponentId, QueryError> {
        let handle = self
            .resources
            .find(self.component_type, component)
            .ok_or(QueryError::ComponentNotFound)?;
        Ok(self.containers.preallocate(handle, self.resources))
    }

    pub fn all(self, components: &[impl ToUID]) -> Result<Self, QueryError> {
        for component in components {
            let id = self.find_component(component)?;
            if self.all.iter().all(|c| *c != component) {
                self.all.push(id);
            }
        }
        Ok(self)
    }

    pub fn any(self, components: &[impl ToUID]) -> Result<Self, QueryError> {
        for component in components {
            let id = self.find_component(component)?;
            if self.any.iter().all(|c| *c != component) {
                self.any.push(component);
            }
        }
        Ok(self)
    }

    pub fn not(self, components: &[impl ToUID]) -> Result<Self, QueryError> {
        for component in components {
            let id = self.find_component(component)?;
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
