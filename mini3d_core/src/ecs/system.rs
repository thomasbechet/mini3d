use alloc::{boxed::Box, vec::Vec};

use crate::{
    slot_map_key,
    utils::{
        slotmap::{Key, SlotMap},
        uid::{ToUID, UID},
    },
};

use super::{
    component::{
        ComponentError, ComponentKey, NativeExclusiveSystem, NativeParallelSystem, System,
        SystemConfig, SystemKind,
    },
    container::ContainerTable,
    entity::{Entity, EntityTable},
    error::ResolverError,
    query::QueryTable,
    view::native::single::NativeSingleViewMut,
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
            .find(component.to_uid())
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

impl SystemInstance {
    pub(crate) fn resolve(
        &mut self,
        config: &SystemConfig,
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
        match self {
            SystemInstance::NativeExclusive(instance) => {
                instance.resolve(config)?;
            }
            SystemInstance::NativeParallel(instance) => {
                instance.resolve(config)?;
            }
            _ => {}
        }
        Ok(())
    }
}

#[derive(Default)]
pub(crate) struct SystemTable {
    pub(crate) instances: SlotMap<SystemKey, SystemInstance>,
    pub(crate) systems: Vec<(UID, Entity)>,
}

impl SystemTable {
    pub(crate) fn add_system(
        &mut self,
        system: &mut System,
        entity: Entity,
        entities: &mut EntityTable,
        queries: &mut QueryTable,
        containers: &mut ContainerTable,
    ) -> Result<(), ComponentError> {
        let uid = system.name.to_uid();
        if self.systems.iter().any(|(key, _)| *key == uid) {
            return Err(ComponentError::DuplicatedEntry);
        }
        let mut instance = match system.kind {
            SystemKind::Native(system) => system.create_instance(),
            _ => unimplemented!(),
        };
        instance.resolve(&system.config, entities, queries, containers)?;
        system.key = self.instances.add(instance);
        Ok(())
    }

    pub(crate) fn remove_system(
        &mut self,
        system: &mut System,
        entity: Entity,
    ) -> Result<(), ComponentError> {
        unimplemented!()
    }
}
