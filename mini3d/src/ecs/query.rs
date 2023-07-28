use crate::{registry::component::ComponentId, utils::slotmap::SparseSecondaryMap};

use super::{archetype::ArchetypeId, container::AnySceneContainer, entity::Entity};

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

pub(crate) struct SpatialIndex {}

pub(crate) struct GraphRelationIndex {}

pub(crate) struct ProbabilityIndex {}

pub(crate) struct GroupQuery {
    archetypes: Vec<ArchetypeId>,
}

pub(crate) struct FilteredQuery {
    sparse: SparseSecondaryMap<u32>,
}

pub type QueryId = usize;

pub(crate) struct QueryTable {
    group_queries: Vec<GroupQuery>,
    filters: Vec<FilteredQuery>,
}

pub struct QueryBuilder<'a>;

impl<'a> QueryBuilder<'a> {
    pub fn all(self, components: &[ComponentId]) -> Self {
        self
    }
    pub fn any(self, components: &[ComponentId]) -> Self {
        self
    }
    pub fn not(self, components: &[ComponentId]) -> Self {
        self
    }
    pub fn filter_added(self) -> Self {
        self
    }
    pub fn filter_removed(self) -> Self {
        self
    }
    pub fn filter_changed(self, components: &[ComponentId]) -> Self {
        self
    }
    pub fn build(self) -> QueryId {
        0
    }
}
