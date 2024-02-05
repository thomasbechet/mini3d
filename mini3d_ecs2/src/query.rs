use crate::{
    bitset::{BitsetMaskIter, IterAnswer},
    container::ComponentId,
    ecs::ECS,
    entity::Entity,
};

#[derive(Default)]
pub struct Query {
    all: [ComponentId; Self::MAX_ALL],
    any: [ComponentId; Self::MAX_ANY],
    not: [ComponentId; Self::MAX_NOT],
    all_size: usize,
    any_size: usize,
    not_size: usize,
}

impl Query {
    pub const MAX_ALL: usize = 8;
    pub const MAX_ANY: usize = 8;
    pub const MAX_NOT: usize = 8;

    pub fn all(mut self, ids: &[ComponentId]) -> Self {
        for (i, id) in ids.iter().enumerate() {
            self.all[i] = *id;
        }
        self.all_size = ids.len();
        self
    }

    pub fn any(mut self, ids: &[ComponentId]) -> Self {
        for (i, id) in ids.iter().enumerate() {
            self.any[i] = *id;
        }
        self.any_size = ids.len();
        self
    }

    pub fn not(mut self, ids: &[ComponentId]) -> Self {
        for (i, id) in ids.iter().enumerate() {
            self.not[i] = *id;
        }
        self.not_size = ids.len();
        self
    }

    pub fn entities<'a>(&'a self, ecs: &'a ECS) -> EntityQuery<'a> {
        EntityQuery::new(ecs, self)
    }
}

pub struct EntityQuery<'a> {
    ecs: &'a ECS<'a>,
    query: &'a Query,
    iter: BitsetMaskIter,
}

impl<'a> EntityQuery<'a> {
    fn fetch_mask(ecs: &ECS, query: &Query, index: usize) -> u32 {
        let mut mask = if query.all_size > 0 {
            ecs.containers.entries[query.all[0]].bitset.mask(index)
        } else {
            0
        };
        for i in 1..query.all_size {
            mask &= ecs.containers.entries[query.all[i]].bitset.mask(index);
        }
        for i in 0..query.any_size {
            mask |= ecs.containers.entries[query.any[i]].bitset.mask(index);
        }
        mask
    }

    pub(crate) fn new(ecs: &'a ECS, query: &'a Query) -> Self {
        Self {
            ecs,
            query,
            iter: BitsetMaskIter::new(Self::fetch_mask(ecs, query, 0)),
        }
    }
}

impl Iterator for EntityQuery<'_> {
    type Item = Entity;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.iter.next() {
                IterAnswer::Some(index) => {
                    let version = self.ecs.entities.version(index);
                    return Some(Entity::new(index, version));
                }
                IterAnswer::None => return None,
                IterAnswer::NewMask(index) => {
                    self.iter
                        .set_mask(Self::fetch_mask(self.ecs, self.query, index));
                    continue;
                }
            }
        }
    }
}
