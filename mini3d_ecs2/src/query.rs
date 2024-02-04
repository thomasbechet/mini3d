use crate::{container::ComponentId, ecs::ECS, entity::Entity};

pub struct EntityQuery<'a> {
    ecs: &'a ECS<'a>,
    all: &'a [ComponentId],
    any: &'a [ComponentId],
    not: &'a [ComponentId],
}

// impl<'a> Query<'a> {}

impl<'a> EntityQuery<'a> {
    pub(crate) fn new(
        ecs: &'a ECS<'a>,
        all: &'a [ComponentId],
        any: &'a [ComponentId],
        not: &'a [ComponentId],
    ) -> Self {
        Self { ecs, all, any, not }
    }
}

impl Iterator for EntityQuery<'_> {
    type Item = Entity;

    fn next(&mut self) -> Option<Self::Item> {
        None
    }
}

// ecs.query().all(&[])
