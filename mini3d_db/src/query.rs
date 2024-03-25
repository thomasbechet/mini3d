use crate::{
    bitset::{BitsetMaskIter, IterAnswer},
    database::{ComponentHandle, Database, GetComponentHandle},
    entity::Entity,
    registry::Registry,
};

#[derive(Default)]
pub struct Query {
    all: [Option<ComponentHandle>; Self::MAX_ALL],
    any: [Option<ComponentHandle>; Self::MAX_ANY],
    not: [Option<ComponentHandle>; Self::MAX_NOT],
    all_size: usize,
    any_size: usize,
    not_size: usize,
}

impl Query {
    pub const MAX_ALL: usize = 8;
    pub const MAX_ANY: usize = 8;
    pub const MAX_NOT: usize = 8;

    pub fn all(mut self, ids: &[impl GetComponentHandle]) -> Self {
        for (i, id) in ids.iter().enumerate() {
            self.all[i] = Some(id.handle());
        }
        self.all_size = ids.len();
        self
    }

    pub fn any(mut self, ids: &[impl GetComponentHandle]) -> Self {
        for (i, id) in ids.iter().enumerate() {
            self.any[i] = Some(id.handle());
        }
        self.any_size = ids.len();
        self
    }

    pub fn not(mut self, ids: &[impl GetComponentHandle]) -> Self {
        for (i, id) in ids.iter().enumerate() {
            self.not[i] = Some(id.handle());
        }
        self.not_size = ids.len();
        self
    }
}

pub struct EntityQuery<'a> {
    query: &'a Query,
    iter: BitsetMaskIter,
}

impl<'a> EntityQuery<'a> {
    fn fetch_mask(registry: &Registry, query: &Query, index: usize) -> u32 {
        let mut mask = if query.all_size > 0 {
            registry.mask(query.all[0].unwrap(), index)
        } else {
            0
        };
        for i in 1..query.all_size {
            mask &= registry.mask(query.all[i].unwrap(), index);
        }
        for i in 0..query.any_size {
            mask |= registry.mask(query.any[i].unwrap(), index);
        }
        mask
    }

    pub(crate) fn new(query: &'a Query, db: &Database) -> Self {
        Self {
            query,
            iter: BitsetMaskIter::new(Self::fetch_mask(&db.registry, query, 0)),
        }
    }

    fn next_entity(&mut self, db: &'a Database) -> Option<Entity> {
        loop {
            match self.iter.next() {
                IterAnswer::Some(index) => {
                    let version = db.registry.entity_version(index);
                    return Some(Entity::new(index, version));
                }
                IterAnswer::None => return None,
                IterAnswer::NewMask(index) => {
                    self.iter
                        .set_mask(Self::fetch_mask(&db.registry, self.query, index));
                    continue;
                }
            }
        }
    }

    pub fn into_iter(self, db: &'a Database) -> EntityQueryIter<'a> {
        EntityQueryIter { query: self, db }
    }
}

pub struct EntityQueryIter<'a> {
    query: EntityQuery<'a>,
    db: &'a Database,
}

impl Iterator for EntityQueryIter<'_> {
    type Item = Entity;

    fn next(&mut self) -> Option<Self::Item> {
        self.query.next_entity(self.db)
    }
}
