use alloc::{boxed::Box, vec::Vec};

use crate::{
    script::component::Program,
    utils::{
        string::AsciiArray,
        uid::{ToUID, UID},
    },
};

use super::{
    component::ComponentKey, container::ContainerTable, context::Context, entity::EntityTable,
    error::ResolverError, query::QueryTable,
};

pub trait ExclusiveSystem: 'static + Default + Clone {
    fn setup(&mut self, _resolver: &mut SystemResolver) -> Result<(), ResolverError> {
        Ok(())
    }
    fn run(&mut self, _ctx: &mut Context) {}
}

pub trait ParallelSystem: 'static + Default + Clone {
    fn setup(&mut self, _resolver: &mut SystemResolver) -> Result<(), ResolverError> {
        Ok(())
    }
    fn run(&mut self, _ctx: &Context) {}
}

pub struct SystemResolver<'a> {
    pub(crate) reads: Vec<ComponentKey>,
    pub(crate) writes: &'a mut Vec<ComponentKey>,
    pub(crate) all: &'a mut Vec<ComponentKey>,
    pub(crate) any: &'a mut Vec<ComponentKey>,
    pub(crate) not: &'a mut Vec<ComponentKey>,
    pub(crate) entities: &'a mut EntityTable,
    pub(crate) queries: &'a mut QueryTable,
    pub(crate) containers: &'a mut ContainerTable,
}

impl<'a> SystemResolver<'a> {
    fn find(&mut self, component: impl ToUID) -> Result<ComponentKey, ResolverError> {
        self.containers
            .find(component)
            .ok_or(ResolverError::ComponentNotFound)
    }

    pub(crate) fn read(&mut self, component: impl ToUID) -> Result<ComponentKey, ResolverError> {
        let key = self.find(component)?;
        if !self.reads.contains(&key) && !self.writes.contains(&key) {
            self.reads.push(key);
        }
        Ok(key)
    }

    pub(crate) fn write(&mut self, component: impl ToUID) -> Result<ComponentKey, ResolverError> {
        let key = self.find(component)?;
        if self.reads.contains(&key) {
            self.reads.retain(|&x| x != key);
        }
        if !self.writes.contains(&key) {
            self.writes.push(key);
        }
        Ok(key)
    }
}

pub(crate) trait AnyNativeExclusiveSystemInstance {
    fn resolve(&mut self, resolver: &mut SystemResolver) -> Result<(), ResolverError>;
    fn run(&self, ctx: &mut Context);
}

pub(crate) trait AnyNativeParallelSystemInstance {
    fn resolve(&mut self, resolver: &mut SystemResolver) -> Result<(), ResolverError>;
    fn run(&self, ctx: &Context);
}

pub(crate) enum ExclusiveSystemInstance {
    Native(Box<dyn AnyNativeExclusiveSystemInstance>),
    Program(Program),
}

impl ExclusiveSystemInstance {
    pub(crate) fn resolve(&mut self, resolver: &mut SystemResolver) -> Result<(), ResolverError> {
        match self {
            Self::Native(instance) => instance.resolve(resolver),
            Self::Program(_) => Ok(()),
        }
    }

    pub(crate) fn run(&self, ctx: &mut Context) {
        match self {
            Self::Native(instance) => instance.run(ctx),
            Self::Program(instance) => {}
        }
    }
}

pub(crate) enum ParallelSystemInstance {
    Native(Box<dyn AnyNativeParallelSystemInstance>),
    Program(Program),
}

impl ParallelSystemInstance {
    pub(crate) fn resolve(&mut self, resolver: &mut SystemResolver) -> Result<(), ResolverError> {
        match self {
            Self::Native(instance) => instance.resolve(resolver),
            Self::Program(_) => Ok(()),
        }
    }

    pub(crate) fn run(&self, ctx: &Context) {
        match self {
            Self::Native(instance) => instance.run(ctx),
            Self::Program(instance) => {}
        }
    }
}

pub(crate) enum SystemInstance {
    Exclusive(ExclusiveSystemInstance),
    Parallel(ParallelSystemInstance),
}

pub(crate) struct SystemInstanceEntry {
    pub(crate) name: AsciiArray<32>,
    pub(crate) set: SystemSetKey,
    pub(crate) stage: SystemStageKey,
    pub(crate) instance: SystemInstance,
    pub(crate) writes: Vec<ComponentKey>,
}

impl SystemInstanceEntry {
    pub(crate) fn setup(
        &mut self,
        entities: &mut EntityTable,
        queries: &mut QueryTable,
        containers: &mut ContainerTable,
    ) -> Result<(), ResolverError> {
        let mut resolver = SystemResolver {
            reads: Vec::new(),
            writes: &mut self.writes,
            all: &mut Default::default(),
            any: &mut Default::default(),
            not: &mut Default::default(),
            entities,
            queries,
            containers,
        };
        match self.instance {
            SystemInstance::Exclusive(ref mut instance) => {
                instance.resolve(&mut resolver)?;
            }
            SystemInstance::Parallel(ref mut instance) => {
                instance.resolve(&mut resolver)?;
            }
        }
        Ok(())
    }
}

#[derive(Default)]
pub(crate) struct SystemTable {
    pub(crate) instances: Vec<SystemInstanceEntry>,
}

impl SystemTable {
    pub(crate) fn find_system_set(&self, uid: UID) -> Option<SystemSetKey> {
        self.sets.iter().find_map(|(key, e)| {
            if e.name.to_uid() == uid {
                Some(key)
            } else {
                None
            }
        })
    }

    pub(crate) fn insert_system_set(
        &mut self,
        set: SystemSetHandle,
        entities: &mut EntityTable,
        queries: &mut QueryTable,
        containers: &mut ContainerTable,
    ) -> Result<(), ResolverError> {
        // Check already existing system set
        if self.instances.iter().any(|instance| instance.set == set) {
            return Ok(());
        }
        // Acquire resource
        resource.increment_ref(set).unwrap();
        let system_set = resource.native::<SystemSet>(set).unwrap();
        // Add instances
        for entry in system_set.0.iter() {
            let system = resource.native::<System>(entry.system).unwrap();
            match &system.kind {
                SystemKind::Native(reflection) => {
                    let instance = reflection.create_instance();
                    self.instances.push(SystemInstanceEntry {
                        set,
                        system: entry.system,
                        stage: entry.stage,
                        instance,
                        writes: Vec::new(),
                    });
                }
                SystemKind::Script(script) => {
                    todo!()
                }
            }
        }
        // Setup instances
        let component_type = resource.find_type(ComponentType::NAME).unwrap();
        for entry in self.instances.iter_mut() {
            entry.setup(entities, queries, containers, resource, component_type)?;
        }
        Ok(())
    }

    pub(crate) fn remove_system_set(&mut self, set: SystemSetHandle) {
        self.instances.retain(|instance| instance.set != set);
    }
}
