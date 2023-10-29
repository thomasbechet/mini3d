use crate::{
    api::Context,
    feature::core::{
        component::{ComponentId, ComponentType},
        system::{System, SystemKind, SystemSet},
    },
    resource::{handle::ResourceHandle, ResourceManager},
    utils::slotmap::{SlotId, SlotMap},
};

use super::{container::ContainerTable, error::ResolverError};

pub(crate) struct SystemInstanceId(pub(crate) SlotId);

pub trait ExclusiveSystem: 'static + Default {
    fn setup(&mut self, resolver: &mut SystemResolver) -> Result<(), ResolverError> {
        Ok(())
    }
    fn run(&self, ctx: &mut Context) {}
}

pub trait ParallelSystem: 'static + Default {
    fn setup(&mut self, resolver: &mut SystemResolver) -> Result<(), ResolverError> {
        Ok(())
    }
    fn run(&self, ctx: &Context) {}
}

use crate::{feature::common::program::Program, utils::uid::ToUID};

use super::{
    entity::EntityTable,
    query::{QueryBuilder, QueryTable},
};

pub struct SystemResolver<'a> {
    system: SystemInstanceId,
    component_type: ResourceHandle,
    reads: Vec<ComponentId>,
    writes: Vec<ComponentId>,
    all: &'a mut Vec<ComponentId>,
    any: &'a mut Vec<ComponentId>,
    not: &'a mut Vec<ComponentId>,
    entities: &'a mut EntityTable,
    queries: &'a mut QueryTable,
    pub(crate) containers: &'a mut ContainerTable,
    pub(crate) resources: &'a mut ResourceManager,
}

impl<'a> SystemResolver<'a> {
    fn find(&mut self, component: impl ToUID) -> Result<ComponentId, ResolverError> {
        let handle = self
            .resources
            .find(self.component_type, component)
            .ok_or(ResolverError::ComponentNotFound)?;
        Ok(self.containers.preallocate(handle, self.resources))
    }

    pub(crate) fn read(&mut self, component: impl ToUID) -> Result<ComponentId, ResolverError> {
        let id = self.find(component)?;
        if !self.reads.contains(&id) && !self.writes.contains(&id) {
            self.reads.push(id);
        }
        Ok(id)
    }

    pub(crate) fn write(&mut self, component: impl ToUID) -> Result<ComponentId, ResolverError> {
        let id = self.find(component)?;
        if self.reads.contains(&id) {
            self.reads.retain(|&x| x != id);
        }
        if !self.writes.contains(&id) {
            self.writes.push(id);
        }
        Ok(id)
    }

    pub fn query(&mut self) -> QueryBuilder<'_> {
        self.all.clear();
        self.any.clear();
        self.not.clear();
        QueryBuilder {
            system: self.system,
            component_type: self.component_type,
            all: self.all,
            any: self.any,
            not: self.not,
            entities: self.entities,
            queries: self.queries,
            containers: self.containers,
            resources: self.resources,
        }
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
    pub(crate) set: ResourceHandle,
    pub(crate) index: usize,
    pub(crate) instance: SystemInstance,
}

impl SystemInstanceEntry {
    pub(crate) fn setup(
        &mut self,
        entities: &mut EntityTable,
        queries: &mut QueryTable,
        containers: &mut ContainerTable,
        resources: &mut ResourceManager,
        component_type: ResourceHandle,
    ) -> Result<(), ResolverError> {
        let mut resolver = SystemResolver {
            system: self.system,
            component_type,
            reads: Vec::new(),
            writes: Vec::new(),
            all: &mut Default::default(),
            any: &mut Default::default(),
            not: &mut Default::default(),
            entities,
            queries,
            containers,
            resources,
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
    pub(crate) sets: Vec<ResourceHandle>,
    pub(crate) instances: SlotMap<SystemInstanceEntry>,
}

impl SystemTable {
    pub(crate) fn insert_system_set(
        &mut self,
        handle: ResourceHandle,
        entities: &mut EntityTable,
        queries: &mut QueryTable,
        containers: &mut ContainerTable,
        resources: &mut ResourceManager,
    ) -> Result<(), ResolverError> {
        // Check existing system set
        if self.sets.iter().find(|e| e.handle() == handle).is_some() {
            return Ok(());
        }
        // Acquire resource
        self.sets.push(resources.increment_ref(handle));
        let set = resources.read::<SystemSet>(handle).unwrap();
        // Add systems
        for (index, entry) in set.0.iter().enumerate() {
            let system = resources.read::<System>(entry.system).unwrap();
            match system.kind {
                SystemKind::Native { reflection } => {
                    let instance = reflection.create_instance();
                    self.instances.add(SystemInstanceEntry {
                        set: handle,
                        index,
                        instance,
                    });
                }
                SystemKind::Script { script } => {
                    todo!()
                }
            }
        }
        // Setup instances
        let component_type = resources.find_type(ComponentType::NAME).unwrap();
        for entry in self.instances.values_mut() {
            entry.setup(entities, queries, containers, resources, component_type)?;
        }
        Ok(())
    }
}
