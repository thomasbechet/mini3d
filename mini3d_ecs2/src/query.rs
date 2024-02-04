use crate::container::ComponentId;

pub struct EntityQuery<'a> {
    all: &'a [ComponentId],
    any: &'a [ComponentId],
    not: &'a [ComponentId],
}

// impl<'a> Query<'a> {}

// ecs.query().all(&[])
