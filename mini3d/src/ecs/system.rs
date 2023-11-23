use crate::{
    api::Context,
    feature::{
        core::resource::ResourceTypeHandle,
        ecs::{
            component::{ComponentKey, ComponentType, ComponentTypeHandle},
            system::{
                System, SystemHandle, SystemKind, SystemSet, SystemSetHandle, SystemStageHandle,
            },
        },
    },
    resource::ResourceManager,
};

use super::{container::ContainerTable, error::ResolverError};

pub trait ExclusiveSystem: 'static + Default + Clone {
    fn setup(&mut self, _resolver: &mut SystemResolver) -> Result<(), ResolverError> {
        Ok(())
    }
    fn run(self, _ctx: &mut Context) {}
}

pub trait ParallelSystem: 'static + Default + Clone {
    fn setup(&mut self, _resolver: &mut SystemResolver) -> Result<(), ResolverError> {
        Ok(())
    }
    fn run(self, _ctx: &Context) {}
}

use crate::{feature::common::program::Program, utils::uid::ToUID};

use super::{entity::EntityTable, query::QueryTable};

pub struct SystemResolver<'a> {
    pub(crate) component_type: ResourceTypeHandle,
    pub(crate) reads: Vec<ComponentKey>,
    pub(crate) writes: &'a mut Vec<ComponentKey>,
    pub(crate) all: &'a mut Vec<ComponentKey>,
    pub(crate) any: &'a mut Vec<ComponentKey>,
    pub(crate) not: &'a mut Vec<ComponentKey>,
    pub(crate) entities: &'a mut EntityTable,
    pub(crate) queries: &'a mut QueryTable,
    pub(crate) containers: &'a mut ContainerTable,
    pub(crate) resources: &'a mut ResourceManager,
}

impl<'a> SystemResolver<'a> {
    fn find(&mut self, component: impl ToUID) -> Result<ComponentKey, ResolverError> {
        let handle = ComponentTypeHandle(
            self.resources
                .find_typed(component, self.component_type)
                .ok_or(ResolverError::ComponentNotFound)?,
        );
        Ok(self.containers.preallocate(handle, self.resources))
    }

    pub(crate) fn read(&mut self, component: impl ToUID) -> Result<ComponentKey, ResolverError> {
        let id = self.find(component)?;
        if !self.reads.contains(&id) && !self.writes.contains(&id) {
            self.reads.push(id);
        }
        Ok(id)
    }

    pub(crate) fn write(&mut self, component: impl ToUID) -> Result<ComponentKey, ResolverError> {
        let id = self.find(component)?;
        if self.reads.contains(&id) {
            self.reads.retain(|&x| x != id);
        }
        if !self.writes.contains(&id) {
            self.writes.push(id);
        }
        Ok(id)
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
    pub(crate) set: SystemSetHandle,
    pub(crate) stage: SystemStageHandle,
    pub(crate) system: SystemHandle,
    pub(crate) instance: SystemInstance,
    pub(crate) writes: Vec<ComponentKey>,
}

impl SystemInstanceEntry {
    pub(crate) fn setup(
        &mut self,
        entities: &mut EntityTable,
        queries: &mut QueryTable,
        containers: &mut ContainerTable,
        resources: &mut ResourceManager,
        component_type: ResourceTypeHandle,
    ) -> Result<(), ResolverError> {
        let mut resolver = SystemResolver {
            component_type,
            reads: Vec::new(),
            writes: &mut self.writes,
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
    pub(crate) instances: Vec<SystemInstanceEntry>,
}

impl SystemTable {
    pub(crate) fn insert_system_set(
        &mut self,
        set: SystemSetHandle,
        entities: &mut EntityTable,
        queries: &mut QueryTable,
        containers: &mut ContainerTable,
        resource: &mut ResourceManager,
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
