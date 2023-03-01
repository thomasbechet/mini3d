
use super::{container::AnyComponentContainer, entity::Entity};

pub struct Query<'a> {
    containers: Vec<&'a dyn AnyComponentContainer>,
}

impl<'a> Query<'a> {

    pub(crate) fn new(containers: Vec<&'a dyn AnyComponentContainer>) -> Self {
        Self { containers }
    }

    pub(crate) fn none() -> Self {
        Self { containers: Vec::new() }
    }

    pub fn iter(&'a self) -> QueryIter<'a> {
        QueryIter {
            query: self,
            index: 0,
            len: self.containers.first().map_or(0, |container| container.len())
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