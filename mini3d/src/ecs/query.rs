use crate::{
    registry::component::ComponentId,
    utils::slotmap::{DenseSlotMap, SlotId, SlotMap},
};

use super::{
    archetype::{ArchetypeId, ArchetypeTable},
    container::AnySceneContainer,
    entity::{Entity, EntityTable},
};

pub struct Query<'a> {
    containers: Vec<&'a dyn AnySceneContainer>,
}

impl<'a> Query<'a> {
    pub(crate) fn new(containers: Vec<&'a dyn AnySceneContainer>) -> Self {
        Self { containers }
    }

    pub(crate) fn none() -> Self {
        Self {
            containers: Vec::new(),
        }
    }

    pub fn iter(&'a self) -> QueryIter<'a> {
        QueryIter {
            query: self,
            index: 0,
            len: self
                .containers
                .first()
                .map_or(0, |container| container.len()),
        }
    }
}

pub struct QueryIter<'a> {
    query: &'a Query<'a>,
    index: usize,
    len: usize,
}

impl<'a> Iterator for QueryIter<'a> {
    type Item = Entity;

    fn next(&mut self) -> Option<Self::Item> {
        while self.index < self.len {
            let entity = self.query.containers[0].entity(self.index);
            self.index += 1;
            let mut valid = true;
            for pool in &self.query.containers[1..] {
                if !pool.contains(entity) {
                    valid = false;
                    break;
                }
            }
            if valid {
                return Some(entity);
            }
        }
        None
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
    }
}

impl<'a> IntoIterator for &'a Query<'a> {
    type Item = Entity;
    type IntoIter = QueryIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

pub struct QueryAdded<'a> {
    containers: Vec<&'a dyn AnySceneContainer>,
}

impl<'a> QueryAdded<'a> {
    pub(crate) fn new(containers: Vec<&'a dyn AnySceneContainer>) -> Self {
        Self { containers }
    }

    pub(crate) fn none() -> Self {
        Self {
            containers: Vec::new(),
        }
    }

    pub fn iter(&'a self) -> QueryAddedIter<'a> {
        QueryAddedIter {
            query: self,
            index: 0,
            len: self
                .containers
                .first()
                .map_or(0, |container| container.len()),
        }
    }
}

pub struct QueryAddedIter<'a> {
    query: &'a QueryAdded<'a>,
    index: usize,
    len: usize,
}

impl<'a> Iterator for QueryAddedIter<'a> {
    type Item = Entity;

    fn next(&mut self) -> Option<Self::Item> {
        while self.index < self.len {
            let entity = self.query.containers[0].entity(self.index);
            self.index += 1;
            let mut valid = true;
            for pool in &self.query.containers[1..] {
                if !pool.contains(entity) {
                    valid = false;
                    break;
                }
            }
            if valid {
                return Some(entity);
            }
        }
        None
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
    }
}

pub struct QueryId(SlotId);
pub struct FilterQueryId(SlotId);

pub(crate) struct QueryEntry {
    archetypes: Vec<ArchetypeId>,
}

pub(crate) struct FilterQueryEntry {
    query: QueryId,
    cycle: usize,
    entities: Vec<Entity>,
}

#[derive(Default)]
pub(crate) struct QueryTable {
    queries: SlotMap<QueryEntry>,
    filter_queries: SlotMap<FilterQueryEntry>,
}

impl QueryTable {
    fn add_query(&mut self, entities: EntityTable, archetypes: &mut ArchetypeTable, group_filters: &[GroupFilter]) -> QueryId {

    }
}

enum GroupFilterKind {
    All,
    Any,
    Not,
}

pub(crate) struct GroupFilter {
    component: ComponentId,
    kind: GroupFilterKind,
}

pub struct QueryBuilder<'a> {
    group_filters: &'a mut Vec<GroupFilter>,
    queries: &'a mut QueryTable,
    archetypes: &'a mut ArchetypeTable,
}

impl<'a> QueryBuilder<'a> {
    fn try_add(&mut self, components: &[ComponentId], filter: GroupFilterKind) {
        for component in components.iter().copied() {
            if self.group_filters
            .iter()
            .all(|filter| filter.component != component) {
                self.group_filters.push(GroupFilter {
                    component,
                    kind: filter,
                });
            }
        }
    }

    pub fn all(mut self, components: &[ComponentId]) -> Self {
        self.try_add(components, GroupFilterKind::All);
        self
    }

    pub fn any(mut self, components: &[ComponentId]) -> Self {
        self.try_add(components, GroupFilterKind::Any);
        self
    }

    pub fn not(mut self, components: &[ComponentId]) -> Self {
        self.try_add(components, GroupFilterKind::Not);
        self
    }

    pub fn build(self) -> QueryId {
        self.queries.add_query(self.archetypes, &self.group_filters)
    }

    pub fn added(self) -> FilterQueryId {
        self.queries.
    }

    pub fn removed(self) -> FilterQueryId {}

    pub fn changed(self, component: ComponentId) -> FilterQueryId {}
}
