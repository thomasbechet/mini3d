use alloc::{boxed::Box, vec::Vec};

use crate::{
    slot_map_key,
    utils::{
        slotmap::SlotMap,
        uid::{ToUID, UID},
    },
};

use super::{
    component::{ComponentError, ComponentKey, NativeExclusiveSystem, NativeParallelSystem},
    container::ContainerTable,
    entity::{Entity, EntityTable},
    error::ResolverError,
    query::QueryTable,
    scheduler::SystemStageKey,
};

slot_map_key!(SystemKey);

pub struct Resolver<'a> {
    pub(crate) reads: Vec<ComponentKey>,
    pub(crate) writes: &'a mut Vec<ComponentKey>,
    pub(crate) all: &'a mut Vec<ComponentKey>,
    pub(crate) any: &'a mut Vec<ComponentKey>,
    pub(crate) not: &'a mut Vec<ComponentKey>,
    pub(crate) entities: &'a mut EntityTable,
    pub(crate) queries: &'a mut QueryTable,
    pub(crate) containers: &'a mut ContainerTable,
}

impl<'a> Resolver<'a> {
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

pub(crate) enum SystemInstance {
    NativeExclusive(Box<dyn NativeExclusiveSystem>),
    NativeParallel(Box<dyn NativeParallelSystem>),
    Script,
}

pub(crate) struct CompiledSystemEntry {
    pub(crate) instance: SystemInstance,
    pub(crate) entity: Entity,
}

impl CompiledSystemEntry {
    pub(crate) fn setup(
        &mut self,
        entities: &mut EntityTable,
        queries: &mut QueryTable,
        containers: &mut ContainerTable,
    ) -> Result<(), ResolverError> {
        let mut resolver = Resolver {
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
    pub(crate) systems: SlotMap<SystemKey, CompiledSystemEntry>,
}

impl SystemTable {
    pub(crate) fn add_system(
        &mut self,
        name: &str,
        entity: Entity,
    ) -> Result<SystemKey, ComponentError> {
        let uid = name.to_uid();
        if self.systems.iter().any(|(x, _)| *x == uid) {
            return Err(ComponentError::DuplicatedEntry);
        }
        Ok(SystemKey::null())
    }

    pub(crate) fn remove_system(&mut self, key: SystemKey) -> Result<(), ComponentError> {
        unimplemented!()
    }

    pub(crate) fn insert_system_set(
        &mut self,
        set: SystemSetHandle,
        entities: &mut EntityTable,
        queries: &mut QueryTable,
        containers: &mut ContainerTable,
    ) -> Result<(), ResolverError> {
        // Check already existing system set
        if self.systems.iter().any(|instance| instance.set == set) {
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
                    self.systems.push(CompiledSystemEntry {
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
        for entry in self.systems.iter_mut() {
            entry.setup(entities, queries, containers, resource, component_type)?;
        }
        Ok(())
    }

    pub(crate) fn remove_system_set(&mut self, set: SystemSetHandle) {
        self.systems.retain(|instance| instance.set != set);
    }
}
