use anyhow::Result;

use super::{context::SystemContext, world::World, query::QueryIter};

struct ParallelSystem<Q: hecs::Query> {
    run: fn(&SystemContext, QueryIter<'_, Q>) -> Result<()>,
}

impl<Q: hecs::Query> ParallelSystem<Q> {
    fn new(run: fn(&SystemContext, QueryIter<'_, Q>) -> Result<()>) -> Self {
        Self { run }
    }
}

pub(crate) trait AnyParallelSystem {
    fn run(&self, context: &SystemContext, world: &World) -> Result<()>;
}

impl <Q: hecs::Query> AnyParallelSystem for ParallelSystem<Q> {
    fn run(&self, context: &SystemContext, world: &World) -> Result<()> {
        (self.run)(context, QueryIter(world.raw_world.query::<Q>().iter()))
    }
}

enum SystemCallback {
    Exclusive(fn(&mut SystemContext, &mut World) -> Result<()>),
    Parallel(Box<dyn AnyParallelSystem>),
}

impl SystemCallback {

    pub fn parallel<Q: hecs::Query + 'static>(callback: fn(&SystemContext, QueryIter<'_, Q>) -> Result<()>) -> Self {
        let callback = ParallelSystem::new(callback);
        Self::Parallel(Box::new(callback))
    }

    pub fn exclusive(callback: fn(&mut SystemContext, &mut World) -> Result<()>) -> Self {
        Self::Exclusive(callback)
    }
}

pub type BuiltinSystem = SystemCallback;

pub(crate) struct BuiltinSystemEntry {
    pub(crate) name: String,
    pub(crate) callback: SystemCallback,
}